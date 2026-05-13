use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{client::SnapdClient, error::Result, types::ChangeId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Interface {
    pub name: String,
    pub summary: Option<String>,
    pub doc_url: Option<String>,
    #[serde(default)]
    pub plugs: Vec<Plug>,
    #[serde(default)]
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plug {
    pub snap: Option<String>,
    pub plug: String,
    pub interface: Option<String>,
    pub attrs: Option<HashMap<String, Value>>,
    pub label: Option<String>,
    #[serde(default)]
    pub connections: Vec<SlotRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub snap: Option<String>,
    pub slot: String,
    pub interface: Option<String>,
    pub attrs: Option<HashMap<String, Value>>,
    pub label: Option<String>,
    #[serde(default)]
    pub connections: Vec<PlugRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub plug: PlugRef,
    pub slot: SlotRef,
    pub interface: Option<String>,
    pub gadget: Option<bool>,
    pub manual: Option<bool>,
    pub slot_attrs: Option<HashMap<String, Value>>,
    pub plug_attrs: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlugRef {
    pub snap: String,
    pub plug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotRef {
    pub snap: String,
    pub slot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connections {
    #[serde(default)]
    pub established: Vec<Connection>,
    #[serde(default)]
    pub undesired: Vec<Connection>,
    #[serde(default)]
    pub plugs: Vec<Plug>,
    #[serde(default)]
    pub slots: Vec<Slot>,
}

impl SnapdClient {
    pub async fn list_interfaces(&self) -> Result<Vec<Interface>> {
        self.get("/v2/interfaces?select=connected").await
    }

    pub async fn list_all_interfaces(&self) -> Result<Vec<Interface>> {
        self.get("/v2/interfaces?plugs=true&slots=true&select=all")
            .await
    }

    pub async fn list_snap_interfaces(&self, snap_name: &str) -> Result<Vec<Interface>> {
        Ok(self
            .list_all_interfaces()
            .await?
            .into_iter()
            .filter(|interface| {
                interface
                    .plugs
                    .iter()
                    .any(|plug| plug.snap.as_deref() == Some(snap_name))
                    || interface
                        .slots
                        .iter()
                        .any(|slot| slot.snap.as_deref() == Some(snap_name))
            })
            .collect())
    }

    pub async fn list_connections(&self) -> Result<Vec<Connection>> {
        let resp: Connections = self.get("/v2/connections").await?;
        Ok(resp.established)
    }

    pub async fn connect_interface(
        &self,
        plug_snap: &str,
        plug_name: &str,
        slot_snap: &str,
        slot_name: &str,
    ) -> Result<ChangeId> {
        self.post_async(
            "/v2/interfaces",
            &json!({
                "action": "connect",
                "plugs": [{ "snap": plug_snap, "plug": plug_name }],
                "slots": [{ "snap": slot_snap, "slot": slot_name }],
            }),
        )
        .await
    }

    pub async fn disconnect_interface(
        &self,
        plug_snap: &str,
        plug_name: &str,
        slot_snap: &str,
        slot_name: &str,
    ) -> Result<ChangeId> {
        self.post_async(
            "/v2/interfaces",
            &json!({
                "action": "disconnect",
                "plugs": [{ "snap": plug_snap, "plug": plug_name }],
                "slots": [{ "snap": slot_snap, "slot": slot_name }],
            }),
        )
        .await
    }
}
