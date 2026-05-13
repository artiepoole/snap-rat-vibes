use serde::{Deserialize, Serialize};
use serde_json::Value;

// --- Wire-protocol types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Sync,
    Async,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSnapdResponse {
    #[serde(rename = "type")]
    pub response_type: ResponseType,
    #[serde(rename = "status-code")]
    pub status_code: u16,
    pub status: String,
    pub result: Option<Value>,
    pub change: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResult {
    pub message: String,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeId(pub String);

/// Snap revision number. Serialized as a string in the snapd API (e.g. `"19"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Revision(pub i64);

impl Serialize for Revision {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Revision {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let n = if let Some(rest) = s.strip_prefix('x') {
            rest.parse::<i64>().map(|n| -n)
        } else {
            s.parse::<i64>()
        }
        .map_err(serde::de::Error::custom)?;
        Ok(Revision(n))
    }
}

// --- Domain enums ---

/// Snap confinement level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SnapConfinement {
    Strict,
    Classic,
    Devmode,
}

/// Snap type (app, kernel, gadget, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SnapType {
    App,
    Kernel,
    Gadget,
    Os,
    Base,
    Core,
    Snapd,
}

/// Installed snap status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SnapStatus {
    Installed,
    Active,
    Available,
    Removed,
}

/// Daemon type for snap services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DaemonType {
    Simple,
    Forking,
    Oneshot,
    Dbus,
    Notify,
}

/// Daemon scope (system-wide or per-user).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DaemonScope {
    System,
    User,
}

/// Alias status kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AliasStatusKind {
    Auto,
    Manual,
    Disabled,
}

/// Status of a change or task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ChangeStatus {
    Do,
    Doing,
    Done,
    Abort,
    Aborting,
    Error,
    Hold,
    Wait,
    Undone,
    Undoing,
}

/// System recovery mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SystemMode {
    Run,
    Recover,
    Install,
}

/// Validation set enforcement mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ValidationSetMode {
    Enforce,
    Monitor,
}

/// Notice type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NoticeType {
    SnapRunInhibit,
    InterfacesRequestsPrompt,
    InterfacesRequestsRuleUpdate,
    ChangeUpdate,
    Warning,
    RefreshInhibit,
}

/// Prompt rule outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PromptOutcome {
    Allow,
    Deny,
}

/// Prompt rule lifespan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PromptLifespan {
    Single,
    Session,
    Forever,
    Timespan,
}
