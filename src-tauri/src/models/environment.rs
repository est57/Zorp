use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Variable {
    pub key: String,
    /// Always empty for secrets when sent to the frontend.
    pub value: String,
    pub enabled: bool,
    pub secret: bool,
    /// For secrets: whether a value is stored in the OS keychain.
    #[serde(default)]
    pub has_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Environment {
    pub id: Uuid,
    pub name: String,
    pub variables: Vec<Variable>,
    pub is_active: bool,
}
