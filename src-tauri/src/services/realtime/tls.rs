use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, aws_lc_rs};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};

use crate::error::{AppError, AppResult};
use crate::models::settings::{ClientCertificate, WorkspaceSettings};

fn tls_error(e: impl std::fmt::Display) -> AppError {
    AppError::Validation(format!("Konfigurasi TLS tidak valid: {e}"))
}

fn read_pem(path: &str, what: &str) -> AppResult<Vec<u8>> {
    std::fs::read(path)
        .map_err(|_| AppError::Validation(format!("File {what} tidak ditemukan: {path}")))
}

/// Accepts any server certificate; used only when the request turns TLS verification off.
#[derive(Debug)]
struct AcceptAnyCertificate(Arc<CryptoProvider>);

impl ServerCertVerifier for AcceptAnyCertificate {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn roots(settings: &WorkspaceSettings) -> AppResult<RootCertStore> {
    let mut store = RootCertStore::empty();
    // Unreadable system certificates are skipped, matching how browsers treat a partial store.
    for cert in rustls_native_certs::load_native_certs().certs {
        let _ = store.add(cert);
    }
    for ca in settings.ca_certificates.iter().filter(|c| c.enabled) {
        let pem = read_pem(&ca.path, "sertifikat CA")?;
        for cert in CertificateDer::pem_slice_iter(&pem) {
            store.add(cert.map_err(tls_error)?).map_err(tls_error)?;
        }
    }
    Ok(store)
}

fn identity(
    cert: &ClientCertificate,
) -> AppResult<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_pem = read_pem(&cert.cert_path, "sertifikat klien")?;
    let key_pem = if cert.key_path.trim().is_empty() {
        cert_pem.clone()
    } else {
        read_pem(&cert.key_path, "private key")?
    };
    let chain = CertificateDer::pem_slice_iter(&cert_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(tls_error)?;
    let key = PrivateKeyDer::from_pem_slice(&key_pem).map_err(tls_error)?;
    Ok((chain, key))
}

/// TLS settings for WebSocket connections, mirroring the HTTP client: system and custom roots plus mTLS.
pub fn client_config(
    settings: &WorkspaceSettings,
    verify_tls: bool,
    client_cert: Option<&ClientCertificate>,
) -> AppResult<Arc<ClientConfig>> {
    let provider = Arc::new(aws_lc_rs::default_provider());
    let builder = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(tls_error)?;
    let builder = if verify_tls {
        builder.with_root_certificates(roots(settings)?)
    } else {
        builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnyCertificate(provider)))
    };
    let config = match client_cert {
        Some(cert) => {
            let (chain, key) = identity(cert)?;
            builder
                .with_client_auth_cert(chain, key)
                .map_err(tls_error)?
        }
        None => builder.with_no_client_auth(),
    };
    Ok(Arc::new(config))
}
