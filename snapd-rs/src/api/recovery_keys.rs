use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{client::SnapdClient, error::Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemRecoveryKeys {
    pub recovery_key: String,
    pub reinstall_key: Option<String>,
}

impl SnapdClient {
    pub async fn get_recovery_keys(&self) -> Result<SystemRecoveryKeys> {
        self.get("/v2/system-recovery-keys").await
    }

    pub async fn remove_recovery_keys(&self) -> Result<()> {
        self.post_sync("/v2/system-recovery-keys", &json!({ "action": "remove" }))
            .await
    }
}
