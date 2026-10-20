use reqwest::{Certificate, Identity, NoProxy, Proxy};

use crate::error::{AppError, AppResult};
use crate::models::settings::{ClientCertificate, ProxySettings, WorkspaceSettings};
use crate::services::request_builder::parse_url;

const PROXY_SCHEMES: [&str; 4] = ["http", "https", "socks5", "socks5h"];
const MAX_CERT_BYTES: u64 = 1024 * 1024;

fn read_small(path: &str, what: &str) -> AppResult<Vec<u8>> {
    let size = std::fs::metadata(path)
        .map_err(|_| AppError::Validation(format!("File {what} tidak ditemukan: {path}")))?
        .len();
    if size > MAX_CERT_BYTES {
        return Err(AppError::Validation(format!(
            "File {what} terlalu besar: {path}"
        )));
    }
    Ok(std::fs::read(path)?)
}

pub fn validate_proxy_url(url: &str) -> AppResult<()> {
    let parsed = url::Url::parse(url.trim()).map_err(|_| {
        AppError::Validation("URL proxy tidak valid. Contoh: http://127.0.0.1:8080".into())
    })?;
    if !PROXY_SCHEMES.contains(&parsed.scheme()) || parsed.host_str().is_none() {
        return Err(AppError::Validation(
            "URL proxy harus diawali http://, https://, socks5://, atau socks5h:// dan memuat host"
                .into(),
        ));
    }
    Ok(())
}

pub fn build_proxy(settings: &ProxySettings, password: Option<&str>) -> AppResult<Option<Proxy>> {
    if !settings.enabled || settings.url.trim().is_empty() {
        return Ok(None);
    }
    validate_proxy_url(&settings.url)?;
    let mut proxy =
        Proxy::all(settings.url.trim()).map_err(|e| AppError::Validation(e.to_string()))?;
    if !settings.username.is_empty() {
        proxy = proxy.basic_auth(&settings.username, password.unwrap_or_default());
    }
    Ok(Some(proxy.no_proxy(NoProxy::from_string(&settings.bypass))))
}

pub fn load_roots(settings: &WorkspaceSettings) -> AppResult<Vec<Certificate>> {
    let mut roots = Vec::new();
    for ca in settings.ca_certificates.iter().filter(|c| c.enabled) {
        let pem = read_small(&ca.path, "sertifikat CA")?;
        let certs = Certificate::from_pem_bundle(&pem).map_err(|_| {
            AppError::Validation(format!("Sertifikat CA bukan PEM yang valid: {}", ca.path))
        })?;
        if certs.is_empty() {
            return Err(AppError::Validation(format!(
                "File tidak berisi sertifikat CA: {}",
                ca.path
            )));
        }
        roots.extend(certs);
    }
    Ok(roots)
}

pub fn load_identity(cert: &ClientCertificate) -> AppResult<Identity> {
    let mut pem = read_small(&cert.cert_path, "sertifikat klien")?;
    if cert.key_path.trim().is_empty() {
        // The certificate file may already contain the private key.
    } else {
        pem.push(b'\n');
        pem.extend(read_small(&cert.key_path, "private key")?);
    }
    Identity::from_pem(&pem).map_err(|_| {
        AppError::Validation(format!(
            "Sertifikat klien untuk {} tidak bisa dibaca. Gunakan PEM dengan private key yang tidak terenkripsi.",
            cert.host
        ))
    })
}

fn host_matches(pattern: &str, host: &str) -> bool {
    let pattern = pattern.trim().to_ascii_lowercase();
    let host = host.to_ascii_lowercase();
    match pattern.strip_prefix("*.") {
        Some(suffix) => host.ends_with(&format!(".{suffix}")),
        None => pattern == host,
    }
}

/// Client certificate that applies to `url`, preferring exact host matches over wildcards.
pub fn certificate_for<'a>(
    settings: &'a WorkspaceSettings,
    url: &str,
) -> Option<&'a ClientCertificate> {
    let host = parse_url(url).ok()?.host_str()?.to_string();
    certificate_for_host(settings, &host)
}

pub fn certificate_for_host<'a>(
    settings: &'a WorkspaceSettings,
    host: &str,
) -> Option<&'a ClientCertificate> {
    let candidates = settings
        .client_certificates
        .iter()
        .filter(|c| c.enabled && host_matches(&c.host, host));
    candidates.max_by_key(|c| !c.host.trim().starts_with("*."))
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;
