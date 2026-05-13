use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{client::SnapdClient, error::Result, types::ChangeId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QuotaCpuValues {
    pub count: Option<u64>,
    pub percentage: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QuotaCpuSetValues {
    #[serde(default)]
    pub cpus: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QuotaJournalValues {
    pub size: Option<u64>,
    pub rate_count: Option<u64>,
    pub rate_period: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QuotaValues {
    pub memory: Option<u64>,
    pub cpu: Option<QuotaCpuValues>,
    pub cpu_set: Option<QuotaCpuSetValues>,
    pub threads: Option<u64>,
    pub journal: Option<QuotaJournalValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QuotaGroup {
    pub group_name: String,
    pub parent: Option<String>,
    #[serde(default)]
    pub subgroups: Vec<String>,
    #[serde(default)]
    pub snaps: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    pub constraints: QuotaValues,
    pub current: Option<QuotaValues>,
}

impl SnapdClient {
    pub async fn list_quotas(&self) -> Result<Vec<QuotaGroup>> {
        self.get("/v2/quotas").await
    }

    pub async fn get_quota(&self, group: &str) -> Result<QuotaGroup> {
        self.get(&format!("/v2/quotas/{group}")).await
    }

    pub async fn ensure_quota(&self, group: &str, constraints: QuotaValues) -> Result<ChangeId> {
        self.post_async(
            "/v2/quotas",
            &json!({
                "action": "ensure",
                "group-name": group,
                "constraints": constraints,
            }),
        )
        .await
    }

    pub async fn remove_quota(&self, group: &str) -> Result<ChangeId> {
        self.post_async(
            "/v2/quotas",
            &json!({
                "action": "remove",
                "group-name": group,
            }),
        )
        .await
    }
}
