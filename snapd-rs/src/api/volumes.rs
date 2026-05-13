use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{client::SnapdClient, error::Result, types::ChangeId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyslotType {
    Recovery,
    Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct KeyslotInfo {
    #[serde(rename = "type")]
    pub type_: KeyslotType,
    #[serde(default)]
    pub roles: Vec<String>,
    pub platform_name: Option<String>,
    pub auth_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemVolumesStructureInfo {
    pub volume_name: String,
    pub name: String,
    pub encrypted: bool,
    #[serde(default)]
    pub keyslots: HashMap<String, KeyslotInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemVolumesResult {
    #[serde(default)]
    pub by_container_role: HashMap<String, SystemVolumesStructureInfo>,
}

#[derive(Debug, Serialize)]
struct SystemVolumesActionRequest<'a, P: Serialize> {
    action: &'a str,
    #[serde(flatten)]
    params: &'a P,
}

impl SnapdClient {
    pub async fn list_system_volumes(&self) -> Result<SystemVolumesResult> {
        self.get("/v2/system-volumes").await
    }

    pub async fn system_volumes_action<P: Serialize>(
        &self,
        action: &str,
        params: &P,
    ) -> Result<ChangeId> {
        self.post_async(
            "/v2/system-volumes",
            &SystemVolumesActionRequest { action, params },
        )
        .await
    }
}
