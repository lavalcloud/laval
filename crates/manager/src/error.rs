use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use anyhow::Error as AnyhowError;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).json(ErrorBody {
            message: self.message.clone(),
        })
    }
}

impl From<AnyhowError> for AppError {
    fn from(err: AnyhowError) -> Self {
        Self::internal(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
