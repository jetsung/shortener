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
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

/// Current user response
#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub username: String,
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

    // Verify password using argon2
    let password_hash = hash_password(&config.password)?;
    if !verify_password(&req.password, &password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let token = generate_token(&req.username)?;

    info!("User logged in successfully: {}", req.username);

    Ok(Json(LoginResponse {
        token,
        username: req.username,
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
    Ok(StatusCode::OK)
}

/// Get current user handler
///
/// GET /api/users/current
pub async fn current_user(
    Extension(user): Extension<User>,
) -> Result<Json<CurrentUserResponse>, AppError> {
    info!("Getting current user: {}", user.username);

    Ok(Json(CurrentUserResponse {
        username: user.username,
    }))
}

/// Hash a password using argon2
fn hash_password(password: &str) -> Result<String, AppError> {
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

/// Verify a password against a hash using argon2
fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
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

/// Generate a JWT token for a user
fn generate_token(username: &str) -> Result<String, AppError> {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: u64,
    }

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 86400; // 24 hours

    let claims = Claims {
        sub: username.to_string(),
        exp: expiration,
    };

    // In production, use a secure secret key from configuration
    let secret = "your-secret-key"; // TODO: Move to config

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))?;

    Ok(token)
}

/// Verify a JWT token and extract the username
pub fn verify_token(token: &str) -> Result<String, AppError> {
    use jsonwebtoken::{DecodingKey, Validation, decode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: u64,
    }

    // In production, use a secure secret key from configuration
    let secret = "your-secret-key"; // TODO: Move to config

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.sub)
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
        let token = generate_token(username).unwrap();

        let verified_username = verify_token(&token).unwrap();
        assert_eq!(verified_username, username);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_handler() {
        let config = Arc::new(AdminConfig {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        });

        let req = LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = login(State(config), Json(req)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.username, "admin");
        assert!(!response.token.is_empty());
    }

    #[tokio::test]
    async fn test_login_invalid_username() {
        let config = Arc::new(AdminConfig {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        });

        let req = LoginRequest {
            username: "wrong".to_string(),
            password: "admin123".to_string(),
        };

        let result = login(State(config), Json(req)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn test_logout_handler() {
        let result = logout().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_current_user_handler() {
        let user = User {
            username: "testuser".to_string(),
        };

        let result = current_user(Extension(user)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.username, "testuser");
    }
}
