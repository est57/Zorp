use std::sync::Arc;

use reqwest::{Certificate, Client, Identity, Proxy, redirect};

use crate::error::{AppError, AppResult};
use crate::services::cookies::CookieJar;
use crate::services::timing::{TimingLayer, TimingResolver};

const MAX_REDIRECTS: usize = 10;

#[derive(Default)]
pub struct ClientOptions {
    pub verify_tls: bool,
    pub follow_redirects: bool,
    pub jar: Option<Arc<CookieJar>>,
    pub proxy: Option<Proxy>,
    pub roots: Vec<Certificate>,
    pub identity: Option<Identity>,
}

/// Minimal client for tests.
#[cfg(test)]
pub fn new_client(
    verify_tls: bool,
    follow_redirects: bool,
    jar: Option<Arc<CookieJar>>,
) -> AppResult<Client> {
    build_client(ClientOptions {
        verify_tls,
        follow_redirects,
        jar,
        ..Default::default()
    })
}

pub fn build_client(options: ClientOptions) -> AppResult<Client> {
    let policy = if options.follow_redirects {
        redirect::Policy::limited(MAX_REDIRECTS)
    } else {
        redirect::Policy::none()
    };
    let mut builder = Client::builder()
        .user_agent(concat!("Zorp/", env!("CARGO_PKG_VERSION")))
        .redirect(policy)
        .tls_danger_accept_invalid_certs(!options.verify_tls)
        .dns_resolver(Arc::new(TimingResolver))
        .connector_layer(TimingLayer);
    if let Some(jar) = options.jar {
        builder = builder.cookie_provider(jar);
    }
    // Without an explicit proxy, reqwest would also read proxy environment variables.
    builder = match options.proxy {
        Some(proxy) => builder.proxy(proxy),
        None => builder.no_proxy(),
    };
    if !options.roots.is_empty() {
        builder = builder.tls_certs_merge(options.roots);
    }
    if let Some(identity) = options.identity {
        builder = builder.identity(identity);
    }
    builder
        .build()
        .map_err(|e| AppError::Validation(format!("Konfigurasi koneksi tidak valid: {e}")))
}
