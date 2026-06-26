use super::{SessionInfo, ThoughtStore};
use crate::types::ThoughtData;
use std::collections::HashMap;
use chrono::Utc;

pub struct MemoryThoughtStore {
    sessions: HashMap<String, Vec<ThoughtData>>,
    created_at: HashMap<String, chrono::DateTime<Utc>>,
    updated_at: HashMap<String, chrono::DateTime<Utc>>,
}

impl MemoryThoughtStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            created_at: HashMap::new(),
            updated_at: HashMap::new(),
        }
    }
}

impl ThoughtStore for MemoryThoughtStore {
    fn save_thought(&mut self, session_id: &str, thought: &ThoughtData) -> Result<(), String> {
        let list = self.sessions.entry(session_id.to_string()).or_insert_with(Vec::new);
        list.push(thought.clone());

        let now = Utc::now();
        self.created_at.entry(session_id.to_string()).or_insert(now);
        self.updated_at.insert(session_id.to_string(), now);
        Ok(())
    }

    fn load_session(&self, session_id: &str) -> Result<Vec<ThoughtData>, String> {
        Ok(self.sessions.get(session_id).cloned().unwrap_or_default())
    }

    fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        let mut list = Vec::new();
        for (id, thoughts) in &self.sessions {
            let created = self.created_at.get(id).copied().unwrap_or_else(Utc::now);
            let updated = self.updated_at.get(id).copied().unwrap_or_else(Utc::now);
            list.push(SessionInfo {
                id: id.clone(),
                created_at: created,
                updated_at: updated,
                total_thoughts: thoughts.len(),
            });
        }
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }

    fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        self.sessions.remove(session_id);
        self.created_at.remove(session_id);
        self.updated_at.remove(session_id);
        Ok(())
    }
}
