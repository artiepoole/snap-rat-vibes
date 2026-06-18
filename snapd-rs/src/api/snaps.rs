use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    client::SnapdClient,
    error::Result,
    types::{ChangeId, DaemonScope, DaemonType, Revision, SnapConfinement, SnapStatus, SnapType},
};

/// Deserialize a component revision that may be the string `"unset"` for
/// components that are available but not yet installed.
fn deserialize_component_revision<'de, D>(d: D) -> std::result::Result<Option<Revision>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    if s == "unset" {
        return Ok(None);
    }
    let n = if let Some(rest) = s.strip_prefix('x') {
        rest.parse::<i64>().map(|n| -n)
    } else {
        s.parse::<i64>()
    }
    .map_err(serde::de::Error::custom)?;
    Ok(Some(Revision(n)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ComponentInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub version: Option<String>,
    /// `None` means the component is available in the store but not installed.
    #[serde(default, deserialize_with = "deserialize_component_revision")]
    pub revision: Option<Revision>,
    pub install_date: Option<String>,
    pub summary: Option<String>,
}

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
    #[serde(default)]
    pub components: Vec<ComponentInfo>,
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

    pub async fn sideload_snap(&self, path: &str) -> Result<ChangeId> {
        self.sideload_snap_inner(path, false).await
    }

    pub async fn sideload_snap_classic(&self, path: &str) -> Result<ChangeId> {
        self.sideload_snap_inner(path, true).await
    }

    async fn sideload_snap_inner(&self, path: &str, classic: bool) -> Result<ChangeId> {
        let data = tokio::fs::read(path).await?;
        let boundary = "snapd-rs-sideload-boundary";
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("snap.snap");

        let mut body = Vec::new();
        // dangerous flag
        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"dangerous\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"true\r\n");
        // classic flag (if needed)
        if classic {
            body.extend_from_slice(b"--");
            body.extend_from_slice(boundary.as_bytes());
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(b"Content-Disposition: form-data; name=\"classic\"\r\n");
            body.extend_from_slice(b"\r\n");
            body.extend_from_slice(b"true\r\n");
        }
        // snap file
        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"snap\"; filename=\"");
        body.extend_from_slice(filename.as_bytes());
        body.extend_from_slice(b"\"\r\n");
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(&data);
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(b"--\r\n");

        let content_type = format!("multipart/form-data; boundary={}", boundary);
        self.post_multipart_async("/v2/snaps", &body, &content_type)
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

    pub async fn remove_snap_purge(&self, name: &str) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{name}"),
            &json!({ "action": "remove", "purge": true }),
        )
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

    pub async fn list_snap_components(&self, snap_name: &str) -> Result<Vec<ComponentInfo>> {
        let snap: Snap = self.get(&format!("/v2/snaps/{snap_name}")).await?;
        Ok(snap.components)
    }

    pub async fn install_snap_component(
        &self,
        snap_name: &str,
        component: &str,
    ) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{snap_name}"),
            &json!({
                "action": "install",
                "components": [component],
            }),
        )
        .await
    }

    pub async fn remove_snap_component(
        &self,
        snap_name: &str,
        component: &str,
    ) -> Result<ChangeId> {
        self.post_async(
            &format!("/v2/snaps/{snap_name}"),
            &json!({
                "action": "remove",
                "components": [component],
            }),
        )
        .await
    }
}
