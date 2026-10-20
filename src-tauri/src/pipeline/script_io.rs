use crate::models::request::{Body, RequestSpec};
use crate::models::response::{ResponseBody, ResponseData};
use crate::services::scripting::{ScriptRequest, ScriptResponse};

pub fn script_request(spec: &RequestSpec) -> ScriptRequest {
    ScriptRequest {
        method: format!("{:?}", spec.method).to_uppercase(),
        url: spec.url.clone(),
        headers: spec
            .headers
            .iter()
            .filter(|h| h.enabled && !h.key.is_empty())
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect(),
        body: match &spec.body {
            Body::Json { text } | Body::Raw { text, .. } => Some(text.clone()),
            Body::Graphql { query, variables } => {
                crate::services::request_builder::graphql_payload(query, variables)
                    .ok()
                    .map(|v| v.to_string())
            }
            _ => None,
        },
    }
}

pub fn script_response(response: &ResponseData) -> ScriptResponse {
    ScriptResponse {
        code: response.status,
        status: response.status_text.clone(),
        headers: response.headers.clone(),
        body: match &response.body {
            ResponseBody::Text { text } => text.clone(),
            ResponseBody::TooLarge { preview, .. } => preview.clone(),
            ResponseBody::Base64 { .. } => String::new(),
        },
        response_time: response.timing.total_ms,
    }
}
