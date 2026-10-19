use serde::{Serialize, Serializer};
use ts_rs::TS;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Kesalahan database: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Migrasi database gagal: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("Kesalahan file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Data tidak valid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0} tidak ditemukan")]
    NotFound(&'static str),
    #[error("{0}")]
    Validation(String),
    #[error("URL tidak valid: {0}")]
    InvalidUrl(String),
    #[error("Variabel tidak ditemukan: {}", .0.join(", "))]
    UnresolvedVariables(Vec<String>),
    #[error("{message}")]
    Network { kind: ErrorKind, message: String },
    #[error("Request dibatalkan")]
    Cancelled,
    #[error("{0}")]
    OAuth(String),
    #[error("Script gagal: {0}")]
    Script(String),
    #[error("Gagal mengakses keychain: {0}")]
    Secret(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ErrorKind {
    Db,
    Io,
    NotFound,
    Validation,
    InvalidUrl,
    UnresolvedVariable,
    Dns,
    Connect,
    Tls,
    Timeout,
    Network,
    Cancelled,
    Secret,
    #[serde(rename = "oauth")]
    OAuth,
    Script,
    Internal,
}

/// Shape of every error returned to the frontend.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct ErrorPayload {
    pub kind: ErrorKind,
    pub message: String,
    pub names: Vec<String>,
}

impl AppError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Db(_) | Self::Migration(_) => ErrorKind::Db,
            Self::Io(_) => ErrorKind::Io,
            Self::Json(_) | Self::Validation(_) => ErrorKind::Validation,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::InvalidUrl(_) => ErrorKind::InvalidUrl,
            Self::UnresolvedVariables(_) => ErrorKind::UnresolvedVariable,
            Self::Network { kind, .. } => *kind,
            Self::Cancelled => ErrorKind::Cancelled,
            Self::OAuth(_) => ErrorKind::OAuth,
            Self::Script(_) => ErrorKind::Script,
            Self::Secret(_) => ErrorKind::Secret,
            Self::Internal(_) => ErrorKind::Internal,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let names = match self {
            Self::UnresolvedVariables(names) => names.clone(),
            _ => Vec::new(),
        };
        ErrorPayload {
            kind: self.kind(),
            message: self.to_string(),
            names,
        }
        .serialize(serializer)
    }
}
