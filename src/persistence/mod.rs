pub mod memory;
pub mod sqlite;

use crate::types::ThoughtData;

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "totalThoughts")]
    pub total_thoughts: usize,
}

#[allow(dead_code)]
pub trait ThoughtStore: Send {
    fn save_thought(&mut self, session_id: &str, thought: &ThoughtData) -> Result<(), String>;
    fn load_session(&self, session_id: &str) -> Result<Vec<ThoughtData>, String>;
    fn list_sessions(&self) -> Result<Vec<SessionInfo>, String>;
    fn delete_session(&mut self, session_id: &str) -> Result<(), String>;
}
