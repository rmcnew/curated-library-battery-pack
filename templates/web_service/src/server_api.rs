use crate::shared::{RequestId, Timestamp, TypeError, error_root_cause};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// API wrapper for error types
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ApiError {
    /// Wrap a TypeError
    #[error("{0}")]
    TypeError(TypeError),
    /// Error due to a bad request argument
    #[error("{0}")]
    BadRequestError(String),
    /// Error due to IO problem
    #[error("{0}")]
    InputOutputError(String),
    /// Error due to resource timeout
    #[error("{0}")]
    ResourceTimeoutError(String),
    /// Error due to bad security configuration
    #[error("{0}")]
    InsecureConfigurationError(String),
}


impl From<TypeError> for ApiError {
    fn from(error: TypeError) -> Self {
        Self::TypeError(error)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        Self::InputOutputError(error.to_string())
    }
}

impl From<url::ParseError> for ApiError {
    fn from(error: url::ParseError) -> Self {
        let error_message = error_root_cause(&error);
        Self::BadRequestError(error_message)
    }
}

impl From<rcgen::Error> for ApiError {
    fn from(error: rcgen::Error) -> Self {
        let error_message = error_root_cause(&error);
        Self::InsecureConfigurationError(error_message)
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        let error_message = error_root_cause(&error);
        // might not be a bad request, see error_message
        Self::BadRequestError(error_message)
    }
}

/// convert error in API execution to server responses
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::TypeError(error) => error.into_response(),
            Self::BadRequestError(_) => {
                let body = format!("{self}");
                let status_code = StatusCode::BAD_REQUEST;
                (status_code, body).into_response()
            }
            Self::InputOutputError(_) => {
                let body = format!("{self}");
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                (status_code, body).into_response()
            }
            Self::ResourceTimeoutError(_) => {
                let body = format!("{self}");
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                (status_code, body).into_response()
            }
            Self::InsecureConfigurationError(_) => {
                let body = format!("{self}");
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                (status_code, body).into_response()
            }
        }
    }
}


/// *** Server Status ***
/// Implement a simple "web service is alive" request and response
pub const STATUS_PATH: &str = "/api/v1/status";
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusRequest {
    pub request_id: RequestId,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResponse {
    pub request: StatusRequest,
    pub timestamp: Timestamp,
}

