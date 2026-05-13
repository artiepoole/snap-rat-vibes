use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{client::SnapdClient, error::Result, types::SystemMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemInfo {
    pub version: String,
    pub series: String,
    pub architecture: String,
    pub build_id: Option<String>,
    pub on_classic: Option<bool>,
    pub managed: Option<bool>,
    pub kernel_version: Option<String>,
    pub system_mode: Option<SystemMode>,
    pub sandbox_features: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FdeStatus {
    Indeterminate,
    Active,
    Inactive,
    Recovery,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutoRepairResult {
    NotInitialized,
    NotAttempted,
    FailedPlatformInit,
    FailedKeyslots,
    FailedEncryptionSupport,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StorageEncryptionStatus {
    pub status: FdeStatus,
    pub auto_repair_result: AutoRepairResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Warning {
    pub message: String,
    pub first_added: String,
    pub last_added: String,
    pub last_shown: Option<String>,
    pub expire_after: Option<String>,
    pub repeat_after: Option<String>,
}

impl SnapdClient {
    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        self.get("/v2/system-info").await
    }

    pub async fn get_storage_encryption_status(&self) -> Result<StorageEncryptionStatus> {
        self.get("/v2/system-info/storage-encrypted").await
    }

    pub async fn get_warnings(&self) -> Result<Vec<Warning>> {
        self.get("/v2/warnings").await
    }

    pub async fn acknowledge_warnings(&self, timestamp: &str) -> Result<()> {
        self.post_sync(
            "/v2/warnings",
            &json!({
                "action": "okay",
                "timestamp": timestamp,
            }),
        )
        .await
    }
}
