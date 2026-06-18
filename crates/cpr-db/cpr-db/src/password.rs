use argon2::{
    password_hash::{PasswordHash, SaltString},
    Argon2,
    PasswordHasher,
    PasswordVerifier,
};
use rand::thread_rng;
use crate::DbResult;

/// Generate a secure Argon2 hash for the given password
pub fn generate_hash(password: &str) -> DbResult<String> {
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| crate::DbError::Password(e.to_string()))?
        .to_string();
    Ok(hash)
}

/// Verify if the given password matches the stored hash
pub fn verify_hash(stored_hash: &str, password: &str) -> DbResult<bool> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| crate::DbError::Password(e.to_string()))?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
