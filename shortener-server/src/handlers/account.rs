use crate::config::AdminConfig;
use crate::errors::AppError;
use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub auto_login: bool,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Current user response
#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub name: String,
}

/// User information extracted from token
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
}

/// Login handler
///
/// POST /api/account/login
pub async fn login(
    State(config): State<Arc<AdminConfig>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    info!("Login attempt for user: {}", req.username);

    // Verify username and password
    if req.username != config.username {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Verify password against the Argon2id hash stored in config
    if !verify_password(&req.password, &config.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let token = generate_token(&req.username)?;

    info!("User logged in successfully: {}", req.username);

    Ok(Json(LoginResponse {
        token,
        error_code: None,
        error_message: None,
    }))
}

/// Logout handler
///
/// POST /api/account/logout
pub async fn logout() -> Result<StatusCode, AppError> {
    info!("User logged out");
    // In a stateless JWT system, logout is typically handled client-side
    // by discarding the token. For a more robust solution, you'd maintain
    // a token blacklist or use refresh tokens.
    Ok(StatusCode::NO_CONTENT)
}

/// Get current user handler
///
/// GET /api/users/current
pub async fn current_user(
    Extension(user): Extension<User>,
) -> Result<Json<CurrentUserResponse>, AppError> {
    info!("Getting current user: {}", user.username);

    Ok(Json(CurrentUserResponse {
        name: user.username,
    }))
}

/// Hash a password using argon2 (Argon2id, PHC string format)
pub fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against an argon2 hash (PHC string format)
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHash, PasswordVerifier},
    };

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Failed to parse password hash: {}", e)))?;

    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generate a JWT for a user (replaces the old in-memory pseudo token)
pub fn generate_token(username: &str) -> Result<String, AppError> {
    let claims = crate::jwt::build_claims(username, None, None);
    crate::jwt::encode(&claims)
}

/// Verify a JWT and extract the username (subject)
pub fn verify_token(token: &str) -> Result<String, AppError> {
    let claims = crate::jwt::decode(token)?;
    Ok(claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_generate_and_verify_token() {
        let username = "testuser";
        let claims = crate::jwt::build_claims(username, None, None);
        let token = crate::jwt::encode_impl(&claims, "test-secret").unwrap();

        let claims = crate::jwt::decode_impl(&token, "test-secret").unwrap();
        assert_eq!(claims.sub, username);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = crate::jwt::decode_impl("invalid.token.here", "test-secret");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_handler() {
        let config = Arc::new(AdminConfig {
            username: "admin".to_string(),
            password_hash: hash_password("admin123").unwrap(),
        });

        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
            auto_login: false,
        };

        let result = login(State(config), Json(req)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(!response.token.is_empty());
        assert!(response.error_code.is_none());
    }

    #[tokio::test]
    async fn test_login_invalid_username() {
        let config = Arc::new(AdminConfig {
            username: "admin".to_string(),
            password_hash: hash_password("admin123").unwrap(),
        });

        let req = LoginRequest {
            username: "wrong".to_string(),
            password: "admin123".to_string(),
            auto_login: false,
        };

        let result = login(State(config), Json(req)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn test_logout_handler() {
        let result = logout().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_current_user_handler() {
        let user = User {
            username: "testuser".to_string(),
        };

        let result = current_user(Extension(user)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.name, "testuser");
    }
}
