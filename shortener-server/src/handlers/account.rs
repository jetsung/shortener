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

    // For now, use simple string comparison for password
    // TODO: In production, store hashed passwords in config and verify against hash
    if req.password != config.password {
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

/// Hash a password using argon2
/// TODO: Use this for production password hashing
#[allow(dead_code)]
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
/// TODO: Use this for production password verification
#[allow(dead_code)]
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

/// Generate a simple token for a user (only lowercase letters and numbers)
pub fn generate_token(username: &str) -> Result<String, AppError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Character set: lowercase letters and numbers only
    const CHARSET: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
    const TOKEN_LENGTH: usize = 32;

    // Generate random token using timestamp and username as seed for simplicity
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Create a simple pseudo-random generator
    let mut seed = timestamp as u64;
    seed ^= username.len() as u64;
    for byte in username.bytes() {
        seed = seed.wrapping_mul(31).wrapping_add(byte as u64);
    }

    let token: String = (0..TOKEN_LENGTH)
        .map(|i| {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let idx = (seed as usize + i) % CHARSET.len();
            CHARSET.chars().nth(idx).unwrap()
        })
        .collect();

    // Store token with user info and expiration (24 hours)
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 86400; // 24 hours

    // Store in global token store
    get_token_store()
        .lock()
        .unwrap()
        .insert(token.clone(), (username.to_string(), expiration));

    Ok(token)
}

/// Verify a token and extract the username
pub fn verify_token(token: &str) -> Result<String, AppError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut store = get_token_store().lock().unwrap();

    if let Some((username, expiration)) = store.get(token) {
        // Check if token is expired
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time > *expiration {
            // Token expired, remove it
            store.remove(token);
            return Err(AppError::Unauthorized("Token expired".to_string()));
        }

        return Ok(username.clone());
    }

    Err(AppError::Unauthorized("Invalid token".to_string()))
}

/// Get the global token store
fn get_token_store() -> &'static std::sync::Mutex<std::collections::HashMap<String, (String, u64)>>
{
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static TOKEN_STORE: OnceLock<Mutex<HashMap<String, (String, u64)>>> = OnceLock::new();
    TOKEN_STORE.get_or_init(|| Mutex::new(HashMap::new()))
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
            password: "admin123".to_string(),
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
