use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GroupBy {
    #[default]
    Tag,
    Path,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OpenApiOptions {
    pub group_by: GroupBy,
    /// Value for the `{{baseUrl}}` collection variable.
    pub base_url: String,
    pub example_bodies: bool,
    pub collection_auth: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OpenApiGroup {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OpenApiSample {
    pub method: String,
    pub name: String,
    pub group: Option<String>,
}

/// What an OpenAPI document contains, shown before anything is imported.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OpenApiSummary {
    pub title: String,
    pub version: String,
    /// For example "OpenAPI 3.0" or "Swagger 2.0".
    pub format: String,
    pub endpoints: u32,
    pub groups: Vec<OpenApiGroup>,
    pub auth: Vec<String>,
    pub servers: Vec<String>,
    pub samples: Vec<OpenApiSample>,
}
