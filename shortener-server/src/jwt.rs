use crate::errors::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode as jwt_decode, encode as jwt_encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT secret environment variable name. Must be set at startup (see main.rs).
pub const JWT_SECRET_ENV: &str = "JWT_SECRET";

/// Path to a file holding the JWT secret. Fallback for `JWT_SECRET`, so that
/// secrets mounted as files (systemd `LoadCredential`, Docker/K8s secrets) can
/// be consumed. When both are set, `JWT_SECRET` wins.
pub const JWT_SECRET_FILE_ENV: &str = "JWT_SECRET_FILE";

/// Claims embedded in the JWT. `sub` is the username/subject, `exp` is the
/// expiry timestamp (seconds since epoch).
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub exp: usize,
}

/// Resolve the JWT secret from `JWT_SECRET`, falling back to the file named by
/// `JWT_SECRET_FILE`. Trailing newlines are stripped from file contents because
/// secret managers commonly append one.
///
/// Returns the failure reason as a plain string so that startup checks in
/// `main.rs` can print it verbatim.
pub fn resolve_secret() -> Result<String, String> {
    if let Ok(secret) = std::env::var(JWT_SECRET_ENV) {
        return Ok(secret);
    }

    let path = std::env::var(JWT_SECRET_FILE_ENV).map_err(|_| {
        format!(
            "{} or {} environment variable is not set",
            JWT_SECRET_ENV, JWT_SECRET_FILE_ENV
        )
    })?;

    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read JWT secret file '{}': {}", path, e))?;

    let secret = raw.trim_end_matches(['\n', '\r']).to_string();
    if secret.is_empty() {
        return Err(format!("JWT secret file '{}' is empty", path));
    }
    Ok(secret)
}

fn secret() -> Result<String, AppError> {
    resolve_secret().map_err(AppError::Internal)
}

/// Encode claims into an HS256 JWT using the `JWT_SECRET` environment variable.
pub fn encode(claims: &Claims) -> Result<String, AppError> {
    let secret = secret()?;
    encode_impl(claims, &secret)
}

/// Internal helper so callers may pass an explicit secret (used by tests).
pub fn encode_impl(claims: &Claims, secret: &str) -> Result<String, AppError> {
    let token = jwt_encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to encode JWT: {}", e)))?;
    Ok(token)
}

/// Decode and validate an HS256 JWT, returning its claims.
pub fn decode(token: &str) -> Result<Claims, AppError> {
    let secret = secret()?;
    decode_impl(token, &secret)
}

/// Internal helper so callers may pass an explicit secret (used by tests).
pub fn decode_impl(token: &str, secret: &str) -> Result<Claims, AppError> {
    let data = jwt_decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;
    Ok(data.claims)
}

/// Build standard claims for a user session (24h expiry).
pub fn build_claims(sub: &str, email: Option<String>, name: Option<String>) -> Claims {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    Claims {
        sub: sub.to_string(),
        email,
        name,
        exp,
    }
}
