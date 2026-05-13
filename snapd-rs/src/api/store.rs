use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    client::SnapdClient,
    error::Result,
    types::{Revision, SnapConfinement, SnapType},
};

use super::snaps::StoreAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ChannelSnapInfo {
    pub revision: Option<Revision>,
    pub confinement: Option<SnapConfinement>,
    pub version: Option<String>,
    pub channel: Option<String>,
    pub size: Option<i64>,
    pub released_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StoreSnap {
    pub id: Option<String>,
    pub name: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub publisher: Option<StoreAccount>,
    pub developer: Option<String>,
    pub version: Option<String>,
    pub channel: Option<String>,
    pub download_size: Option<i64>,
    pub revision: Option<Revision>,
    #[serde(default)]
    pub channels: HashMap<String, ChannelSnapInfo>,
    #[serde(rename = "type")]
    pub type_: Option<SnapType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
}

impl SnapdClient {
    pub async fn find_snaps(&self, query: &str) -> Result<Vec<StoreSnap>> {
        self.get(&format!("/v2/find?q={query}")).await
    }

    pub async fn find_snap_by_name(&self, name: &str) -> Result<Option<StoreSnap>> {
        let mut snaps: Vec<StoreSnap> = self.get(&format!("/v2/find?name={name}")).await?;
        Ok(snaps.drain(..).next())
    }

    pub async fn list_categories(&self) -> Result<Vec<Category>> {
        self.get("/v2/categories").await
    }
}
