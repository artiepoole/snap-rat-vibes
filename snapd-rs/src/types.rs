use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use strum::{Display, EnumIter, EnumString, IntoStaticStr, VariantNames};

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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr, EnumIter, VariantNames,
)]
#[strum(serialize_all = "kebab-case")]
#[non_exhaustive]
pub enum NoticeType {
    SnapRunInhibit,
    InterfacesRequestsPrompt,
    InterfacesRequestsRuleUpdate,
    ChangeUpdate,
    Warning,
    RefreshInhibit,
}

impl NoticeType {
    pub fn as_str(&self) -> &'static str {
        (*self).into()
    }
}

impl Serialize for NoticeType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NoticeType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(|_| {
            serde::de::Error::unknown_variant(&value, <NoticeType as strum::VariantNames>::VARIANTS)
        })
    }
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

#[cfg(test)]
mod tests {
    use super::NoticeType;
    use strum::IntoEnumIterator;

    #[test]
    fn notice_type_variants_constant_matches_enum_iteration() {
        let iterated_variants: Vec<_> = NoticeType::iter()
            .map(|notice_type| notice_type.as_str())
            .collect();

        assert_eq!(
            iterated_variants,
            <NoticeType as strum::VariantNames>::VARIANTS
        );
    }

    #[test]
    fn notice_type_display_and_serde_cover_all_variants() {
        for notice_type in NoticeType::iter() {
            let expected = notice_type.as_str();

            assert_eq!(notice_type.to_string(), expected);
            assert_eq!(
                serde_json::to_string(&notice_type).unwrap(),
                format!("\"{expected}\"")
            );

            let deserialized: NoticeType =
                serde_json::from_str(&format!("\"{expected}\"")).unwrap();
            assert_eq!(deserialized, notice_type);
        }
    }
}
