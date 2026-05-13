use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    client::SnapdClient,
    error::Result,
    types::{ChangeId, SystemMode},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemModelData {
    pub model: String,
    pub brand_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreAccount {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub validation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAction {
    pub title: String,
    pub mode: SystemMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RecoverySystem {
    pub current: bool,
    pub default_recovery_system: bool,
    pub label: String,
    pub model: SystemModelData,
    pub brand: StoreAccount,
    #[serde(default)]
    pub actions: Vec<SystemAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemDetails {
    pub current: bool,
    pub label: String,
    pub model: HashMap<String, Value>,
    pub brand: StoreAccount,
    #[serde(default)]
    pub actions: Vec<SystemAction>,
    #[serde(default)]
    pub volumes: HashMap<String, Value>,
    pub storage_encryption: Option<Value>,
    pub available_optional: Option<AvailableOptional>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AvailableOptional {
    #[serde(default)]
    pub snaps: Vec<String>,
    #[serde(default)]
    pub components: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListSystemsResponse {
    #[serde(default)]
    systems: Vec<RecoverySystem>,
}

impl SnapdClient {
    pub async fn list_systems(&self) -> Result<Vec<RecoverySystem>> {
        let response: ListSystemsResponse = self.get("/v2/systems").await?;
        Ok(response.systems)
    }

    pub async fn get_system(&self, label: &str) -> Result<SystemDetails> {
        self.get(&format!("/v2/systems/{label}")).await
    }

    pub async fn reboot_into_system(&self, label: &str, mode: SystemMode) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/systems/{label}"),
            &json!({ "action": "reboot", "mode": mode }),
        )
        .await
    }
}
