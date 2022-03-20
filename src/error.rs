use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorCode {
    MissingEventHeader,
    InvalidSignature,
    InvalidUserAgent,
    MalformedEventHeader,
    UnsupportedEventHeader(String),
    MalformedEventBody(#[from] serde_json::Error),
    MalformedEventBodyField(String, String),
    UnhandledError(String),
}

#[derive(Serialize)]
pub struct ErrorCodeDetail {
    #[serde(skip)]
    status_code: StatusCode,
    internal_code: u32,
    message: String,
}

impl ErrorCode {
    pub fn details(&self) -> ErrorCodeDetail {
        self.into()
    }
}

impl ErrorCodeDetail {
    pub fn with_status_code<T: Into<String>>(
        status_code: StatusCode,
        internal_code: u32,
        message: T,
    ) -> Self {
        Self {
            internal_code,
            status_code,
            message: message.into(),
        }
    }

    pub fn bad_request<T: Into<String>>(internal_code: u32, message: T) -> Self {
        Self::with_status_code(StatusCode::BAD_REQUEST, internal_code, message)
    }

    pub fn server_error<T: Into<String>>(internal_code: u32, message: T) -> Self {
        Self::with_status_code(StatusCode::INTERNAL_SERVER_ERROR, internal_code, message)
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn internal_code(&self) -> u32 {
        self.internal_code
    }
}

impl From<&ErrorCode> for ErrorCodeDetail {
    fn from(value: &ErrorCode) -> Self {
        match value {
            ErrorCode::MissingEventHeader => Self::bad_request(1, "Missing X-GitHub-Event header"),
            ErrorCode::InvalidSignature => {
                Self::bad_request(2, "Invalid X-Hub-Signature-256 signature")
            }
            ErrorCode::InvalidUserAgent => Self::bad_request(3, "Invalid User-Agent"),
            ErrorCode::MalformedEventHeader => Self::bad_request(4, "Malformed event header"),
            ErrorCode::UnsupportedEventHeader(event) => {
                Self::bad_request(5, format!("Unsupported event header: '{}'", event))
            }
            ErrorCode::MalformedEventBody(e) => {
                Self::bad_request(6, format!("Malformed event body: '{}'", e))
            }
            ErrorCode::MalformedEventBodyField(field, e) => Self::bad_request(
                7,
                format!("Malformed event body field '{}': '{}'", field, e),
            ),
            ErrorCode::UnhandledError(e) => {
                Self::server_error(99, format!("Unhandled error: '{}'", e))
            }
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let detail = ErrorCodeDetail::from(self);
        f.write_str(&detail.message)
    }
}
