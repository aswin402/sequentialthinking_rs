use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::Stream;
use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::server::SequentialThinkingServer;
use crate::tools::ToolRegistry;
use crate::transport::{handle_message, Transport};

/// State shared between HTTP request handlers.
#[derive(Clone)]
struct AppState {
    server: Arc<Mutex<SequentialThinkingServer>>,
    tool_registry: Arc<ToolRegistry>,
    sessions: Arc<Mutex<HashMap<String, mpsc::Sender<serde_json::Value>>>>,
}

/// A wrapper stream that cleans up the session in the sessions map when dropped.
struct SseSessionStream<S> {
    inner: S,
    sessions: Arc<Mutex<HashMap<String, mpsc::Sender<serde_json::Value>>>>,
    session_id: String,
}

impl<S: Stream> Stream for SseSessionStream<S> {
    type Item = S::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Safe because we are not moving the inner stream
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.poll_next(cx)
    }
}

impl<S> Drop for SseSessionStream<S> {
    fn drop(&mut self) {
        let sessions = Arc::clone(&self.sessions);
        let session_id = self.session_id.clone();
        tokio::spawn(async move {
            let mut lock = sessions.lock().await;
            if lock.remove(&session_id).is_some() {
                tracing::info!(session_id = %session_id, "SSE connection dropped, session cleaned up");
            }
        });
    }
}

pub struct HttpTransport {
    pub port: u16,
}

impl Transport for HttpTransport {
    async fn run(
        self,
        server: Arc<Mutex<SequentialThinkingServer>>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Result<(), String> {
        let port = self.port;
        let state = AppState {
            server,
            tool_registry,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = Router::new()
            .route("/sse", get(sse_handler))
            .route("/message", post(message_handler))
            .route("/health", get(health_handler))
            .layer(tower_http::cors::CorsLayer::permissive())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

        tracing::info!("Sequential Thinking MCP Server running on HTTP/SSE at http://0.0.0.0:{}", port);

        axum::serve(listener, app)
            .await
            .map_err(|e| format!("HTTP server error: {}", e))?;

        Ok(())
    }
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel(100);

    // Register the session sender
    {
        let mut sessions = state.sessions.lock().await;
        sessions.insert(session_id.clone(), tx.clone());
    }

    tracing::info!(session_id = %session_id, "New SSE connection established");

    // Immediately send the endpoint event over the connection
    let endpoint_url = format!("/message?sessionId={}", session_id);
    let _ = tx.send(serde_json::json!({
        "__mcp_internal_event__": "endpoint",
        "data": endpoint_url
    })).await;

    // Convert channel receiver to Stream of Event
    let stream = ReceiverStream::new(rx).map(move |val| {
        if let Some(event_name) = val.get("__mcp_internal_event__") {
            let name = event_name.as_str().unwrap_or("message");
            let data = val.get("data").unwrap().as_str().unwrap_or("");
            Event::default().event(name).data(data)
        } else {
            Event::default().data(val.to_string())
        }
    });

    let session_stream = SseSessionStream {
        inner: stream,
        sessions: Arc::clone(&state.sessions),
        session_id,
    };

    Sse::new(session_stream.map(Ok)).keep_alive(KeepAlive::default())
}

#[derive(serde::Deserialize)]
struct MessageParams {
    #[serde(rename = "sessionId")]
    session_id: String,
}

async fn message_handler(
    State(state): State<AppState>,
    Query(params): Query<MessageParams>,
    body: String,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    let session_id = params.session_id;

    // Retrieve sender for this session
    let tx = {
        let sessions = state.sessions.lock().await;
        sessions.get(&session_id).cloned()
    };

    let tx = match tx {
        Some(sender) => sender,
        None => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                format!("Session not found: {}", session_id),
            ));
        }
    };

    // Route request through handle_message
    let mut server = state.server.lock().await;
    if let Some(response) = handle_message(&body, &state.tool_registry, &mut server) {
        if let Err(e) = tx.send(response).await {
            tracing::error!(error = %e, "Failed to send response back to client over SSE");
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send response over SSE".to_string(),
            ));
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "message": "Sequential Thinking MCP Server is healthy"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;
    use crate::persistence::memory::MemoryThoughtStore;

    fn setup_test_app() -> Router {
        let store = Box::new(MemoryThoughtStore::new());
        let server = SequentialThinkingServer::new(store, true);
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(crate::tools::sequentialthinking::SequentialThinkingTool));

        let state = AppState {
            server: Arc::new(Mutex::new(server)),
            tool_registry: Arc::new(registry),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        };

        Router::new()
            .route("/sse", get(sse_handler))
            .route("/message", post(message_handler))
            .route("/health", get(health_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = setup_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_sse_and_message_flow() {
        let app = setup_test_app();

        // 1. Establish SSE stream
        let sse_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(sse_response.status(), StatusCode::OK);
        
        // 2. Read the first event from the SSE stream to extract the session ID
        let mut stream = sse_response.into_body().into_data_stream();
        let chunk = stream.next().await.unwrap().unwrap();
        let body_str = String::from_utf8(chunk.to_vec()).unwrap();
        
        assert!(body_str.contains("event: endpoint"));
        
        // Find session ID from the endpoint data string
        let session_marker = "sessionId=";
        let pos = body_str.find(session_marker).expect("Session ID should be in SSE body");
        let session_id_raw = &body_str[pos + session_marker.len()..];
        // Extract session ID until any whitespace/newlines
        let session_id = session_id_raw.split_whitespace().next().unwrap().trim();

        // 3. Send tools/list request to /message?sessionId=...
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        });

        let msg_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/message?sessionId={}", session_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(msg_response.status(), StatusCode::OK);
    }
}
