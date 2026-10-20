//! Assertions and variable rules that work on a response without writing a script.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AssertOp {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    NotContains,
    Matches,
    Exists,
    NotExists,
    /// `value` names a JSON type: string, number, boolean, object, array or null.
    IsType,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Assertion {
    pub enabled: bool,
    /// For example `res.status` or `res.body.user.id`.
    pub expr: String,
    pub op: AssertOp,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum VarPhase {
    /// Set before sending, for this request only.
    Pre,
    /// Read from the response and stored.
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum VarTarget {
    #[default]
    Collection,
    Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VarRule {
    pub enabled: bool,
    pub phase: VarPhase,
    pub name: String,
    /// A value for `pre`, or a response expression such as `res.body.token` for `post`.
    pub expr: String,
    #[serde(default)]
    pub target: VarTarget,
}
