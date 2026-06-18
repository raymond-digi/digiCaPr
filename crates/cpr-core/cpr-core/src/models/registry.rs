use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Registry value types supported by the key-value store
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum RegistryValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Json(serde_json::Value),
}

impl RegistryValue {
    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            RegistryValue::String(_) => "String",
            RegistryValue::Integer(_) => "Integer",
            RegistryValue::Boolean(_) => "Boolean",
            RegistryValue::Json(_) => "Json",
        }
    }

    /// Create a RegistryValue from a string and type
    pub fn from_string_and_type(value: &str, value_type: &str) -> crate::Result<Self> {
        match value_type {
            "String" => Ok(RegistryValue::String(value.to_string())),
            "Integer" => {
                let int_val = value.parse::<i64>().map_err(|_| {
                    crate::PayrollError::ValidationError(format!(
                        "Invalid integer value: {}",
                        value
                    ))
                })?;
                Ok(RegistryValue::Integer(int_val))
            }
            "Boolean" => {
                let bool_val = value.parse::<bool>().map_err(|_| {
                    crate::PayrollError::ValidationError(format!(
                        "Invalid boolean value: {}",
                        value
                    ))
                })?;
                Ok(RegistryValue::Boolean(bool_val))
            }
            "Json" => {
                let json_val: serde_json::Value = serde_json::from_str(value).map_err(|e| {
                    crate::PayrollError::ValidationError(format!(
                        "Invalid JSON value: {}",
                        e
                    ))
                })?;
                Ok(RegistryValue::Json(json_val))
            }
            _ => Err(crate::PayrollError::ValidationError(format!(
                "Unknown value type: {}",
                value_type
            ))),
        }
    }

    /// Get the string representation of the value
    pub fn to_string_value(&self) -> String {
        match self {
            RegistryValue::String(s) => s.clone(),
            RegistryValue::Integer(i) => i.to_string(),
            RegistryValue::Boolean(b) => b.to_string(),
            RegistryValue::Json(j) => j.to_string(),
        }
    }
}

/// Registry entry representing a key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: Option<i64>,
    pub key_path: String,
    pub value: RegistryValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RegistryEntry {
    /// Create a new registry entry
    pub fn new(key_path: String, value: RegistryValue) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            key_path,
            value,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate the registry entry
    pub fn validate(&self) -> crate::Result<()> {
        if self.key_path.trim().is_empty() {
            return Err(crate::PayrollError::ValidationError(
                "Key path cannot be empty".to_string(),
            ));
        }

        // Validate key path format (alphanumeric, underscores, hyphens, slashes)
        if !self
            .key_path
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '/' || c == '.')
        {
            return Err(crate::PayrollError::ValidationError(
                "Key path can only contain alphanumeric characters, underscores, hyphens, slashes, and dots".to_string(),
            ));
        }

        Ok(())
    }
}
