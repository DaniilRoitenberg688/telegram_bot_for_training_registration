use std::{fmt::Display};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ServiceError {
    NotFound,
    Error { error: String },
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "cannot find object"),
            Self::Error { error } => write!(f, "unexpected error: {}", error)
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Error {
                error: value.to_string(),
            },
        }
    }
}

