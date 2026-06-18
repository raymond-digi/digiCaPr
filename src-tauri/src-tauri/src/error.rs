use serde::Serialize;

/// Error type for Tauri commands
/// Converts various error types into a serializable format for the frontend
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl CommandError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// Convert from cpr-core PayrollError
impl From<cpr_core::error::PayrollError> for CommandError {
    fn from(err: cpr_core::error::PayrollError) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

// Convert from cpr-db DbError
impl From<cpr_db::DbError> for CommandError {
    fn from(err: cpr_db::DbError) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

// Convert from rusqlite Error
impl From<rusqlite::Error> for CommandError {
    fn from(err: rusqlite::Error) -> Self {
        CommandError {
            message: format!("Database error: {}", err),
        }
    }
}

// Convert from std::io::Error
impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError {
            message: format!("IO error: {}", err),
        }
    }
}

// Convert from String
impl From<String> for CommandError {
    fn from(err: String) -> Self {
        CommandError { message: err }
    }
}

// Convert from &str
impl From<&str> for CommandError {
    fn from(err: &str) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

// Convert from Box<dyn StdError + Send + Sync>
impl From<Box<dyn std::error::Error + Send + Sync>> for CommandError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}
