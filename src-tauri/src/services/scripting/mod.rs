use std::collections::HashMap;
use std::time::{Duration, Instant};

use rquickjs::{Context, Runtime};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::models::script::{LogEntry, TestResult};

const TIME_LIMIT: Duration = if cfg!(test) {
    Duration::from_millis(300)
} else {
    Duration::from_secs(5)
};
const MEMORY_LIMIT: usize = 64 * 1024 * 1024;
const STACK_LIMIT: usize = 1024 * 1024;
const SOURCE: &str = concat!(
    include_str!("expect.js"),
    "\n",
    include_str!("prelude.js"),
    "\n",
    include_str!("epilogue.js")
);

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ScriptKind {
    #[serde(rename = "pre")]
    PreRequest,
    #[serde(rename = "test")]
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScriptRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptResponse {
    pub code: u16,
    pub status: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub response_time: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptInput {
    pub kind: ScriptKind,
    pub script: String,
    pub request_name: String,
    pub request: ScriptRequest,
    pub response: Option<ScriptResponse>,
    /// Combined values as they resolve for this request.
    pub variables: HashMap<String, String>,
    pub environment: HashMap<String, String>,
    pub collection_variables: HashMap<String, String>,
    pub has_environment: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptOutput {
    pub env: HashMap<String, String>,
    pub env_unset: Vec<String>,
    pub coll: HashMap<String, String>,
    pub coll_unset: Vec<String>,
    /// Values set with pm.variables.set, applied to this request only.
    pub locals: HashMap<String, String>,
    pub tests: Vec<TestResult>,
    pub logs: Vec<LogEntry>,
    pub error: Option<String>,
    pub request: ScriptRequest,
}

impl ScriptOutput {
    pub fn changes_variables(&self) -> bool {
        !(self.env.is_empty()
            && self.env_unset.is_empty()
            && self.coll.is_empty()
            && self.coll_unset.is_empty())
    }
}

/// Runs a user script in an isolated QuickJS runtime with time, memory and stack limits.
pub fn run(input: &ScriptInput) -> AppResult<ScriptOutput> {
    let runtime = Runtime::new().map_err(|e| AppError::Internal(e.to_string()))?;
    runtime.set_memory_limit(MEMORY_LIMIT);
    runtime.set_max_stack_size(STACK_LIMIT);
    let deadline = Instant::now() + TIME_LIMIT;
    runtime.set_interrupt_handler(Some(Box::new(move || Instant::now() > deadline)));
    let context = Context::full(&runtime).map_err(|e| AppError::Internal(e.to_string()))?;
    let input_json = serde_json::to_string(input)?;

    let output: Result<String, String> = context.with(|ctx| {
        ctx.globals()
            .set("__input_json", input_json)
            .map_err(|e| e.to_string())?;
        ctx.eval::<String, _>(SOURCE).map_err(|_| {
            let exception = ctx.catch();
            exception
                .as_exception()
                .and_then(|e| e.message())
                .unwrap_or_else(|| "Script dihentikan".to_string())
        })
    });

    match output {
        Ok(json) => Ok(serde_json::from_str(&json)?),
        Err(_) if Instant::now() > deadline => Err(AppError::Script(
            "Script melebihi batas waktu 5 detik".into(),
        )),
        Err(message) => Err(AppError::Script(message)),
    }
}

#[cfg(test)]
mod tests;
