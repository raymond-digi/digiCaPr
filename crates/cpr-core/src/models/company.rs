use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Province;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Option<i64>,
    pub name: String,
    pub business_number: Option<String>, // CRA Business Number (BN)
    pub address: String,
    pub province: Province,
    pub created_at: DateTime<Utc>,
}

impl Company {
    pub fn new(name: String, province: Province) -> Self {
        Self {
            id: Some(1), // Single company per database
            name,
            business_number: None,
            address: String::new(),
            province,
            created_at: Utc::now(),
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::PayrollError::ValidationError(
                "Company name cannot be empty".to_string()
            ));
        }

        // Validate business number format if provided (9 digits + 2 letter program identifier + 4 digit reference)
        if let Some(ref bn) = self.business_number {
            let bn_clean = bn.replace(" ", "").replace("-", "");
            if !bn_clean.is_empty() {
                // Accept either:
                // 1. Just 9 digits (base BN): 123456789
                // 2. Full format (9 digits + RP/RC/RT + 4 digits): 123456789RP0001
                let is_valid = if bn_clean.len() == 9 {
                    bn_clean.chars().all(|c| c.is_ascii_digit())
                } else if bn_clean.len() == 15 {
                    // 9 digits + 2 letters + 4 digits
                    bn_clean[0..9].chars().all(|c| c.is_ascii_digit()) &&
                    bn_clean[9..11].chars().all(|c| c.is_ascii_alphabetic()) &&
                    bn_clean[11..15].chars().all(|c| c.is_ascii_digit())
                } else {
                    false
                };
                
                if !is_valid {
                    return Err(crate::PayrollError::ValidationError(
                        "Business Number must be either 9 digits (e.g., 123456789) or full format (e.g., 123456789RP0001)".to_string()
                    ));
                }
            }
        }

        Ok(())
    }
}
