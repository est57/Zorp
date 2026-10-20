use std::time::{Duration, Instant};

use tokio::runtime::Handle;

use crate::services::client_pool::ClientKey;
use crate::services::http_engine::classify;
use crate::services::request_builder::parse_url;
use crate::services::scripting::{HostRequest, ScriptHost, ScriptResponse};
use crate::state::AppState;

const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;

/// Sends `pm.sendRequest` calls with the workspace connection settings. Script runs on a
/// blocking thread, so each call waits on the async runtime it was created from.
pub struct PipelineHost {
    client: reqwest::Client,
    runtime: Handle,
}

impl PipelineHost {
    pub fn new(state: &AppState) -> Option<Self> {
        let key = ClientKey {
            verify_tls: true,
            follow_redirects: true,
            use_cookies: true,
            identity: None,
        };
        Some(Self {
            client: state.client(key).ok()?,
            runtime: Handle::try_current().ok()?,
        })
    }

    async fn fetch(&self, request: HostRequest) -> Result<ScriptResponse, String> {
        let url = parse_url(&request.url).map_err(|e| e.to_string())?;
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .map_err(|_| format!("Method {} tidak valid", request.method))?;
        let mut builder = self.client.request(method, url).timeout(TIMEOUT);
        for (key, value) in &request.headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
        if let Some(body) = request.body {
            builder = builder.body(body);
        }
        let started = Instant::now();
        let mut response = builder.send().await.map_err(|e| classify(e).to_string())?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| classify(e).to_string())?
        {
            if bytes.len() + chunk.len() > MAX_BODY_BYTES {
                return Err("Respons pm.sendRequest melebihi 5 MB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(ScriptResponse {
            code: response.status().as_u16(),
            status: response
                .status()
                .canonical_reason()
                .unwrap_or_default()
                .to_string(),
            headers: response
                .headers()
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        String::from_utf8_lossy(v.as_bytes()).into_owned(),
                    )
                })
                .collect(),
            body: String::from_utf8_lossy(&bytes).into_owned(),
            response_time: started.elapsed().as_millis() as u32,
        })
    }
}

impl ScriptHost for PipelineHost {
    fn send(&self, request: HostRequest) -> Result<ScriptResponse, String> {
        self.runtime.block_on(self.fetch(request))
    }
}
