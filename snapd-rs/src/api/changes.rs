use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{client::SnapdClient, error::Result, types::ChangeStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub status: ChangeStatus,
    #[serde(rename = "spawn-time")]
    pub spawn_time: Option<String>,
    #[serde(rename = "ready-time")]
    pub ready_time: Option<String>,
    #[serde(default)]
    pub tasks: Vec<Task>,
    pub err: Option<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub status: ChangeStatus,
    pub progress: TaskProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub done: i64,
    pub total: i64,
}

impl SnapdClient {
    pub async fn list_changes(&self) -> Result<Vec<Change>> {
        self.get("/v2/changes").await
    }

    pub async fn list_all_changes(&self) -> Result<Vec<Change>> {
        self.get("/v2/changes?select=all").await
    }

    pub async fn get_change(&self, id: &str) -> Result<Change> {
        self.get(&format!("/v2/changes/{id}")).await
    }

    pub async fn abort_change(&self, id: &str) -> Result<Change> {
        self.post_sync(&format!("/v2/changes/{id}"), &json!({ "action": "abort" }))
            .await
    }
}
