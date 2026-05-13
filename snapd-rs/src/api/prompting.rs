use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{
    client::SnapdClient,
    error::Result,
    types::{PromptLifespan, PromptOutcome},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Interface {
    Home,
    Camera,
    AudioRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HomePromptConstraints {
    pub path: String,
    #[serde(default)]
    pub requested_permissions: Vec<String>,
    #[serde(default)]
    pub available_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmptyPromptConstraints {
    #[serde(default)]
    pub requested_permissions: Vec<String>,
    #[serde(default)]
    pub available_permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PromptConstraints {
    Home(HomePromptConstraints),
    Empty(EmptyPromptConstraints),
}

impl PromptConstraints {
    fn from_interface_and_value(
        interface: &Interface,
        constraints: Value,
    ) -> std::result::Result<Self, serde_json::Error> {
        match interface {
            Interface::Home => serde_json::from_value(constraints).map(PromptConstraints::Home),
            Interface::Camera | Interface::AudioRecord => {
                serde_json::from_value(constraints).map(PromptConstraints::Empty)
            }
        }
    }

    fn to_value_for_interface(
        &self,
        interface: &Interface,
    ) -> std::result::Result<Value, serde_json::Error> {
        match (interface, self) {
            (Interface::Home, PromptConstraints::Home(constraints)) => {
                serde_json::to_value(constraints)
            }
            (Interface::Camera | Interface::AudioRecord, PromptConstraints::Empty(constraints)) => {
                serde_json::to_value(constraints)
            }
            (Interface::Home, PromptConstraints::Empty(_)) => {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "constraints do not match interface home",
                )))
            }
            (Interface::Camera | Interface::AudioRecord, PromptConstraints::Home(_)) => {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "constraints do not match marker interface",
                )))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub id: String,
    pub timestamp: String,
    pub snap: String,
    pub pid: i32,
    pub cgroup: String,
    pub interface: Interface,
    pub constraints: PromptConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonPrompt {
    id: String,
    timestamp: String,
    snap: String,
    pid: i32,
    cgroup: String,
    interface: Interface,
    constraints: Value,
}

impl<'de> Deserialize<'de> for Prompt {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = JsonPrompt::deserialize(deserializer)?;
        let constraints =
            PromptConstraints::from_interface_and_value(&wire.interface, wire.constraints)
                .map_err(serde::de::Error::custom)?;
        Ok(Prompt {
            id: wire.id,
            timestamp: wire.timestamp,
            snap: wire.snap,
            pid: wire.pid,
            cgroup: wire.cgroup,
            interface: wire.interface,
            constraints,
        })
    }
}

impl Serialize for Prompt {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let constraints = self
            .constraints
            .to_value_for_interface(&self.interface)
            .map_err(serde::ser::Error::custom)?;
        JsonPrompt {
            id: self.id.clone(),
            timestamp: self.timestamp.clone(),
            snap: self.snap.clone(),
            pid: self.pid,
            cgroup: self.cgroup.clone(),
            interface: self.interface.clone(),
            constraints,
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PermissionEntry {
    pub outcome: PromptOutcome,
    pub lifespan: PromptLifespan,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RulePermissionEntry {
    pub outcome: PromptOutcome,
    pub lifespan: PromptLifespan,
    pub expiration: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuleConstraintsHome {
    pub path_pattern: String,
    #[serde(default)]
    pub permissions: HashMap<String, RulePermissionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuleConstraintsEmpty {
    #[serde(default)]
    pub permissions: HashMap<String, RulePermissionEntry>,
}

#[derive(Debug, Clone)]
pub enum RuleConstraints {
    Home(RuleConstraintsHome),
    Empty(RuleConstraintsEmpty),
}

impl RuleConstraints {
    fn from_interface_and_value(
        interface: &Interface,
        constraints: Value,
    ) -> std::result::Result<Self, serde_json::Error> {
        match interface {
            Interface::Home => serde_json::from_value(constraints).map(RuleConstraints::Home),
            Interface::Camera | Interface::AudioRecord => {
                serde_json::from_value(constraints).map(RuleConstraints::Empty)
            }
        }
    }

    fn to_value_for_interface(
        &self,
        interface: &Interface,
    ) -> std::result::Result<Value, serde_json::Error> {
        match (interface, self) {
            (Interface::Home, RuleConstraints::Home(constraints)) => {
                serde_json::to_value(constraints)
            }
            (Interface::Camera | Interface::AudioRecord, RuleConstraints::Empty(constraints)) => {
                serde_json::to_value(constraints)
            }
            (Interface::Home, RuleConstraints::Empty(_)) => {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "rule constraints do not match interface home",
                )))
            }
            (Interface::Camera | Interface::AudioRecord, RuleConstraints::Home(_)) => {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "rule constraints do not match marker interface",
                )))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub timestamp: String,
    pub user: u32,
    pub snap: String,
    pub interface: Interface,
    pub constraints: RuleConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuleJson {
    id: String,
    timestamp: String,
    user: u32,
    snap: String,
    interface: Interface,
    constraints: Value,
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RuleJson::deserialize(deserializer)?;
        let constraints =
            RuleConstraints::from_interface_and_value(&wire.interface, wire.constraints)
                .map_err(serde::de::Error::custom)?;
        Ok(Rule {
            id: wire.id,
            timestamp: wire.timestamp,
            user: wire.user,
            snap: wire.snap,
            interface: wire.interface,
            constraints,
        })
    }
}

impl Serialize for Rule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let constraints = self
            .constraints
            .to_value_for_interface(&self.interface)
            .map_err(serde::ser::Error::custom)?;
        RuleJson {
            id: self.id.clone(),
            timestamp: self.timestamp.clone(),
            user: self.user,
            snap: self.snap.clone(),
            interface: self.interface.clone(),
            constraints,
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HomePromptReplyConstraints {
    pub path_pattern: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmptyPromptReplyConstraints {
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptReplyConstraints {
    Home(HomePromptReplyConstraints),
    Empty(EmptyPromptReplyConstraints),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddRuleConstraintsHome {
    pub path_pattern: String,
    #[serde(default)]
    pub permissions: HashMap<String, PermissionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddRuleConstraintsEmpty {
    #[serde(default)]
    pub permissions: HashMap<String, PermissionEntry>,
}

#[derive(Debug, Clone)]
pub enum AddRuleConstraints {
    Home(AddRuleConstraintsHome),
    Empty(AddRuleConstraintsEmpty),
}

impl AddRuleConstraints {
    fn from_interface_and_value(
        interface: &Interface,
        constraints: Value,
    ) -> std::result::Result<Self, serde_json::Error> {
        match interface {
            Interface::Home => serde_json::from_value(constraints).map(AddRuleConstraints::Home),
            Interface::Camera | Interface::AudioRecord => {
                serde_json::from_value(constraints).map(AddRuleConstraints::Empty)
            }
        }
    }

    fn to_value_for_interface(
        &self,
        interface: &Interface,
    ) -> std::result::Result<Value, serde_json::Error> {
        match (interface, self) {
            (Interface::Home, AddRuleConstraints::Home(constraints)) => {
                serde_json::to_value(constraints)
            }
            (
                Interface::Camera | Interface::AudioRecord,
                AddRuleConstraints::Empty(constraints),
            ) => serde_json::to_value(constraints),
            (Interface::Home, AddRuleConstraints::Empty(_)) => {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "add-rule constraints do not match interface home",
                )))
            }
            (Interface::Camera | Interface::AudioRecord, AddRuleConstraints::Home(_)) => {
                Err(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "add-rule constraints do not match marker interface",
                )))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddRuleContents {
    pub snap: String,
    pub interface: Interface,
    pub constraints: AddRuleConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddRuleContentsJson {
    snap: String,
    interface: Interface,
    constraints: Value,
}

impl<'de> Deserialize<'de> for AddRuleContents {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AddRuleContentsJson::deserialize(deserializer)?;
        let constraints =
            AddRuleConstraints::from_interface_and_value(&wire.interface, wire.constraints)
                .map_err(serde::de::Error::custom)?;
        Ok(AddRuleContents {
            snap: wire.snap,
            interface: wire.interface,
            constraints,
        })
    }
}

impl Serialize for AddRuleContents {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let constraints = self
            .constraints
            .to_value_for_interface(&self.interface)
            .map_err(serde::ser::Error::custom)?;
        AddRuleContentsJson {
            snap: self.snap.clone(),
            interface: self.interface.clone(),
            constraints,
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveRulesSelector {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<Interface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuleConstraintsPatchHome {
    pub path_pattern: Option<String>,
    pub permissions: Option<HashMap<String, Option<PermissionEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuleConstraintsPatchEmpty {
    pub permissions: Option<HashMap<String, Option<PermissionEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleConstraintsPatch {
    Home(RuleConstraintsPatchHome),
    Empty(RuleConstraintsPatchEmpty),
}

#[derive(Debug, Clone, Serialize)]
struct PostPromptRequestBody {
    action: PromptOutcome,
    lifespan: PromptLifespan,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<String>,
    constraints: PromptReplyConstraints,
}

#[derive(Debug, Clone, Serialize)]
struct PostRulesRequestBody {
    action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule: Option<AddRuleContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<RemoveRulesSelector>,
}

#[derive(Debug, Clone, Serialize)]
struct PatchRuleContents {
    constraints: RuleConstraintsPatch,
}

#[derive(Debug, Clone, Serialize)]
struct PostRuleRequestBody {
    action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule: Option<PatchRuleContents>,
}

impl SnapdClient {
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>> {
        self.get("/v2/interfaces/requests/prompts").await
    }

    pub async fn get_prompt(&self, id: &str) -> Result<Prompt> {
        self.get(&format!("/v2/interfaces/requests/prompts/{id}"))
            .await
    }

    pub async fn reply_to_prompt(
        &self,
        id: &str,
        action: PromptOutcome,
        lifespan: PromptLifespan,
        duration: Option<String>,
        constraints: PromptReplyConstraints,
    ) -> Result<Vec<String>> {
        let request = PostPromptRequestBody {
            action,
            lifespan,
            duration,
            constraints,
        };
        self.post_sync(&format!("/v2/interfaces/requests/prompts/{id}"), &request)
            .await
    }

    pub async fn list_prompt_rules(&self) -> Result<Vec<Rule>> {
        self.get("/v2/interfaces/requests/rules").await
    }

    pub async fn add_prompt_rule(&self, rule: AddRuleContents) -> Result<Rule> {
        let request = PostRulesRequestBody {
            action: "add",
            rule: Some(rule),
            selector: None,
        };
        self.post_sync("/v2/interfaces/requests/rules", &request)
            .await
    }

    pub async fn remove_prompt_rules(&self, selector: RemoveRulesSelector) -> Result<Vec<Rule>> {
        let request = PostRulesRequestBody {
            action: "remove",
            rule: None,
            selector: Some(selector),
        };
        self.post_sync("/v2/interfaces/requests/rules", &request)
            .await
    }

    pub async fn get_prompt_rule(&self, id: &str) -> Result<Rule> {
        self.get(&format!("/v2/interfaces/requests/rules/{id}"))
            .await
    }

    pub async fn patch_prompt_rule(
        &self,
        id: &str,
        constraints: RuleConstraintsPatch,
    ) -> Result<Rule> {
        let request = PostRuleRequestBody {
            action: "patch",
            rule: Some(PatchRuleContents { constraints }),
        };
        self.post_sync(&format!("/v2/interfaces/requests/rules/{id}"), &request)
            .await
    }

    pub async fn remove_prompt_rule(&self, id: &str) -> Result<Rule> {
        self.post_sync(
            &format!("/v2/interfaces/requests/rules/{id}"),
            &PostRuleRequestBody {
                action: "remove",
                rule: None,
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reply_to_prompt_request_uses_action_field() {
        let request = serde_json::to_value(PostPromptRequestBody {
            action: PromptOutcome::Allow,
            lifespan: PromptLifespan::Session,
            duration: None,
            constraints: PromptReplyConstraints::Empty(EmptyPromptReplyConstraints {
                permissions: vec!["read".to_string()],
            }),
        })
        .unwrap();

        assert_eq!(
            request,
            json!({
                "action": "allow",
                "lifespan": "session",
                "constraints": { "permissions": ["read"] },
            })
        );
    }

    #[test]
    fn add_prompt_rule_request_uses_action_wrapper() {
        let request = serde_json::to_value(PostRulesRequestBody {
            action: "add",
            rule: Some(AddRuleContents {
                snap: "test-snap".to_string(),
                interface: Interface::Home,
                constraints: AddRuleConstraints::Home(AddRuleConstraintsHome {
                    path_pattern: "/tmp/**".to_string(),
                    permissions: HashMap::new(),
                }),
            }),
            selector: None,
        })
        .unwrap();

        assert_eq!(
            request,
            json!({
                "action": "add",
                "rule": {
                    "snap": "test-snap",
                    "interface": "home",
                    "constraints": {
                        "path-pattern": "/tmp/**",
                        "permissions": {},
                    },
                },
            })
        );
    }

    #[test]
    fn remove_prompt_rules_request_uses_selector_wrapper() {
        let request = serde_json::to_value(PostRulesRequestBody {
            action: "remove",
            rule: None,
            selector: Some(RemoveRulesSelector {
                snap: Some("test-snap".to_string()),
                interface: None,
            }),
        })
        .unwrap();

        assert_eq!(
            request,
            json!({
                "action": "remove",
                "selector": {
                    "snap": "test-snap",
                },
            })
        );
    }

    #[test]
    fn patch_prompt_rule_request_uses_action_wrapper() {
        let request = serde_json::to_value(PostRuleRequestBody {
            action: "patch",
            rule: Some(PatchRuleContents {
                constraints: RuleConstraintsPatch::Empty(RuleConstraintsPatchEmpty {
                    permissions: Some(HashMap::new()),
                }),
            }),
        })
        .unwrap();

        assert_eq!(
            request,
            json!({
                "action": "patch",
                "rule": {
                    "constraints": {
                        "permissions": {},
                    },
                },
            })
        );
    }

    #[test]
    fn remove_prompt_rule_request_uses_action_wrapper() {
        let request = serde_json::to_value(PostRuleRequestBody {
            action: "remove",
            rule: None,
        })
        .unwrap();
        assert_eq!(request, json!({ "action": "remove" }));
    }

    #[test]
    fn prompt_constraints_are_deserialized_using_interface() {
        let prompt: Prompt = serde_json::from_value(json!({
            "id": "1",
            "timestamp": "2026-01-01T00:00:00Z",
            "snap": "test-snap",
            "pid": 1234,
            "cgroup": "/foo",
            "interface": "camera",
            "constraints": {
                "requested-permissions": ["access"],
                "available-permissions": ["access"]
            }
        }))
        .unwrap();

        assert_eq!(prompt.interface, Interface::Camera);
        assert!(matches!(prompt.constraints, PromptConstraints::Empty(_)));
    }

    #[test]
    fn prompt_home_constraints_are_deserialized_using_interface() {
        let prompt: Prompt = serde_json::from_value(json!({
            "id": "2",
            "timestamp": "2026-01-01T00:00:00Z",
            "snap": "test-snap",
            "pid": 4321,
            "cgroup": "/bar",
            "interface": "home",
            "constraints": {
                "path": "/home/test/file.txt",
                "requested-permissions": ["read"],
                "available-permissions": ["read", "write", "execute"]
            }
        }))
        .unwrap();

        match prompt.constraints {
            PromptConstraints::Home(home) => {
                assert_eq!(home.path, "/home/test/file.txt");
                assert_eq!(home.requested_permissions, vec!["read"]);
            }
            PromptConstraints::Empty(_) => panic!("expected home constraints"),
        }
    }

    #[test]
    fn rule_constraints_are_deserialized_using_interface() {
        let rule: Rule = serde_json::from_value(json!({
            "id": "3",
            "timestamp": "2026-01-01T00:00:00Z",
            "user": 1000,
            "snap": "test-snap",
            "interface": "home",
            "constraints": {
                "path-pattern": "/home/test/**",
                "permissions": {
                    "read": {
                        "outcome": "allow",
                        "lifespan": "forever"
                    }
                }
            }
        }))
        .unwrap();

        match rule.constraints {
            RuleConstraints::Home(home) => {
                assert_eq!(home.path_pattern, "/home/test/**");
                assert!(home.permissions.contains_key("read"));
            }
            RuleConstraints::Empty(_) => panic!("expected home rule constraints"),
        }
    }

    #[test]
    fn rule_empty_constraints_are_deserialized_using_interface() {
        let rule: Rule = serde_json::from_value(json!({
            "id": "4",
            "timestamp": "2026-01-01T00:00:00Z",
            "user": 1000,
            "snap": "test-snap",
            "interface": "camera",
            "constraints": {
                "permissions": {
                    "access": {
                        "outcome": "allow",
                        "lifespan": "forever"
                    }
                }
            }
        }))
        .unwrap();

        match rule.constraints {
            RuleConstraints::Empty(empty) => {
                assert!(empty.permissions.contains_key("access"));
            }
            RuleConstraints::Home(_) => panic!("expected empty rule constraints"),
        }
    }

    #[test]
    fn add_rule_contents_constraints_are_deserialized_using_interface() {
        let add_rule: AddRuleContents = serde_json::from_value(json!({
            "snap": "test-snap",
            "interface": "home",
            "constraints": {
                "path-pattern": "/home/test/**",
                "permissions": {
                    "read": {
                        "outcome": "allow",
                        "lifespan": "timespan",
                        "duration": "10m"
                    }
                }
            }
        }))
        .unwrap();

        match add_rule.constraints {
            AddRuleConstraints::Home(home) => {
                assert_eq!(home.path_pattern, "/home/test/**");
                assert!(home.permissions.contains_key("read"));
            }
            AddRuleConstraints::Empty(_) => panic!("expected home add-rule constraints"),
        }
    }

    #[test]
    fn add_rule_contents_empty_constraints_are_deserialized_using_interface() {
        let add_rule: AddRuleContents = serde_json::from_value(json!({
            "snap": "test-snap",
            "interface": "audio-record",
            "constraints": {
                "permissions": {
                    "access": {
                        "outcome": "allow",
                        "lifespan": "forever"
                    }
                }
            }
        }))
        .unwrap();

        match add_rule.constraints {
            AddRuleConstraints::Empty(empty) => {
                assert!(empty.permissions.contains_key("access"));
            }
            AddRuleConstraints::Home(_) => panic!("expected empty add-rule constraints"),
        }
    }

    #[test]
    fn add_rule_constraints_must_match_interface() {
        let request = AddRuleContents {
            snap: "test-snap".to_string(),
            interface: Interface::Home,
            constraints: AddRuleConstraints::Empty(AddRuleConstraintsEmpty {
                permissions: HashMap::new(),
            }),
        };
        assert!(serde_json::to_value(request).is_err());
    }
}
