use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("No available backend for model: {0}")]
    NoAvailableBackend(String),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Server error: {0}")]
    #[allow(dead_code)]
    Server(String),

    #[error("Upstream error: {status} - {message}")]
    Upstream {
        status: u16,
        message: String,
        /// 上游返回的 Retry-After 时长（毫秒），用于重试退避。
        retry_after_ms: Option<u64>,
    },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error", self.to_string()),
            AppError::ProviderNotFound(_) => (StatusCode::NOT_FOUND, "provider_not_found", self.to_string()),
            AppError::ModelNotFound(_) => (StatusCode::NOT_FOUND, "model_not_found", self.to_string()),
            AppError::NoAvailableBackend(_) => {
                (StatusCode::BAD_GATEWAY, "no_available_backend", self.to_string())
            }
            AppError::Upstream { status, message, .. } => (
                StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY),
                "upstream_error",
                message.clone(),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", self.to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "type": code,
                "param": null,
                "code": code
            }
        }));

        (status, body).into_response()
    }
}

impl From<keyring::Error> for AppError {
    fn from(err: keyring::Error) -> Self {
        AppError::Keyring(err.to_string())
    }
}
