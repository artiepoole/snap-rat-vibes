use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    client::SnapdClient,
    error::Result,
    types::{ChangeId, DaemonScope, DaemonType, Revision, SnapConfinement, SnapStatus, SnapType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StoreAccount {
    pub id: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub validation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Snap {
    pub id: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub installed_size: Option<u64>,
    pub install_date: Option<String>,
    pub name: String,
    pub publisher: Option<StoreAccount>,
    pub developer: Option<String>,
    pub status: Option<SnapStatus>,
    #[serde(rename = "type")]
    pub type_: Option<SnapType>,
    pub version: Option<String>,
    pub channel: Option<String>,
    pub tracking_channel: Option<String>,
    pub revision: Option<Revision>,
    pub confinement: Option<SnapConfinement>,
    pub devmode: Option<bool>,
    pub jailmode: Option<bool>,
    pub trymode: Option<bool>,
    pub private: Option<bool>,
    pub broken: Option<String>,
    pub contact: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub apps: Vec<SnapApp>,
    #[serde(default)]
    pub screenshots: Vec<Screenshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SnapApp {
    pub snap: Option<String>,
    pub name: String,
    pub desktop_file: Option<String>,
    pub daemon: Option<DaemonType>,
    pub daemon_scope: Option<DaemonScope>,
    pub enabled: Option<bool>,
    pub active: Option<bool>,
    pub common_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub url: Option<String>,
    pub width: Option<u64>,
    pub height: Option<u64>,
}

impl SnapdClient {
    pub async fn list_snaps(&self) -> Result<Vec<Snap>> {
        self.get("/v2/snaps").await
    }

    pub async fn get_snap(&self, name: &str) -> Result<Snap> {
        self.get(&format!("/v2/snaps/{name}")).await
    }

    pub async fn install_snap(&self, name: &str, channel: Option<&str>) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{name}"),
            &json!({
                "action": "install",
                "channel": channel,
            }),
        )
        .await
    }

    pub async fn install_snap_classic(
        &self,
        name: &str,
        channel: Option<&str>,
    ) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{name}"),
            &json!({
                "action": "install",
                "channel": channel,
                "classic": true,
            }),
        )
        .await
    }

    pub async fn remove_snap(&self, name: &str) -> Result<ChangeId> {
        self.post_async(&format!("/v2/snaps/{name}"), &json!({ "action": "remove" }))
            .await
    }

    pub async fn refresh_snap(&self, name: &str, channel: Option<&str>) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{name}"),
            &json!({
                "action": "refresh",
                "channel": channel,
            }),
        )
        .await
    }

    pub async fn revert_snap(&self, name: &str) -> Result<ChangeId> {
        self.post_async(&format!("/v2/snaps/{name}"), &json!({ "action": "revert" }))
            .await
    }

    pub async fn enable_snap(&self, name: &str) -> Result<ChangeId> {
        self.post_async(&format!("/v2/snaps/{name}"), &json!({ "action": "enable" }))
            .await
    }

    pub async fn disable_snap(&self, name: &str) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{name}"),
            &json!({ "action": "disable" }),
        )
        .await
    }

    pub async fn get_snap_conf(&self, name: &str, keys: &[&str]) -> Result<Value> {
        let path = if keys.is_empty() {
            format!("/v2/snaps/{name}/conf")
        } else {
            format!("/v2/snaps/{name}/conf?keys={}", keys.join(","))
        };
        self.get(&path).await
    }

    pub async fn set_snap_conf(&self, name: &str, conf: Value) -> Result<ChangeId> {
        self.put(&format!("/v2/snaps/{name}/conf"), &conf).await
    }
}
