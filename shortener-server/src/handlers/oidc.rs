use crate::config::OidcConfig;
use crate::errors::AppError;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use openidconnect::{
    AuthType, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl,
    Scope, OAuth2TokenResponse,
    core::{CoreClient, CoreGenderClaim, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client, AuthenticationFlow, EmptyAdditionalClaims, TokenResponse,
};
use std::sync::Arc;

/// Query parameters returned by the IdP to `GET /api/oidc/callback`.
#[derive(Debug, serde::Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
}

/// Build an OIDC client from the configured issuer via discovery.
///
/// The `redirect_url` must be the exact callback URL registered with the IdP.
/// Since `redirect_uri` is no longer a config option, it is derived from the
/// `Host` header of the current request (see [`build_redirect_url`]).
async fn build_client(
    config: &OidcConfig,
    redirect_url: RedirectUrl,
) -> Result<CoreClient, AppError> {
    if !config.enabled {
        return Err(AppError::NotFound(
            "OIDC login is not enabled".to_string(),
        ));
    }
    let issuer = config
        .issuer
        .as_ref()
        .ok_or_else(|| AppError::Internal("OIDC issuer is not configured".to_string()))?;

    let issuer_url = IssuerUrl::new(issuer.clone())
        .map_err(|e| AppError::Internal(format!("Invalid OIDC issuer URL: {}", e)))?;

    let metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|e| {
            // openidconnect 的 Display 只输出 `Request failed` 等短消息，
            // 这里把完整错误链（含底层 reqwest / TLS 原因）拼进日志便于排查。
            let mut chain = String::new();
            let mut source: Option<&dyn std::error::Error> = Some(&e);
            while let Some(err) = source {
                if !chain.is_empty() {
                    chain.push_str(" <- ");
                }
                chain.push_str(&err.to_string());
                source = err.source();
            }
            AppError::Internal(format!("OIDC discovery failed: {}", chain))
        })?;

    let client_id = ClientId::new(
        config
            .client_id
            .clone()
            .ok_or_else(|| AppError::Internal("OIDC client_id is not configured".to_string()))?,
    );

    let client_secret = config.client_secret.clone().map(ClientSecret::new);

    let client = CoreClient::from_provider_metadata(metadata, client_id, client_secret)
        .set_redirect_uri(redirect_url)
        .set_auth_type(AuthType::BasicAuth);

    Ok(client)
}

/// Derive the OIDC callback URL from the current request:
/// `{scheme}://{host}/api/oidc/callback`.
///
/// The `Host` header is always present on HTTP/1.1 requests, and the scheme
/// honors an `X-Forwarded-Proto` header when the service sits behind a TLS
/// terminating reverse proxy (falls back to `https`).
fn build_redirect_url(headers: &HeaderMap) -> Result<RedirectUrl, AppError> {
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Internal("Missing Host header for OIDC callback URL".to_string()))?;

    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("https");

    RedirectUrl::new(format!("{}://{}/api/oidc/callback", scheme, host))
        .map_err(|e| AppError::Internal(format!("Invalid OIDC callback URL: {}", e)))
}

/// `GET /api/oidc/login`
///
/// Redirects the browser to the IdP authorization endpoint. The nonce is
/// stashed in the OAuth `state` so the callback can verify the id_token.
/// 登录成功后固定跳转前端 `/#/dashboard`（写死在 state 中，不依赖前端传参）。
pub async fn oidc_login(
    State(config): State<Arc<OidcConfig>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let redirect_url = build_redirect_url(&headers)?;
    let client = build_client(&config, redirect_url).await?;

    let nonce = Nonce::new_random();
    // State format: `r:<redirect>|n:<nonce>`；redirect 固定为前端 dashboard 页。
    let state = CsrfToken::new(format!("r:#/dashboard|n:{}", nonce.secret()));

    let (auth_url, _csrf_token, _nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            || state,
            || nonce,
        )
        // 注意：openidconnect 库在 authorize_url 内部已自动附加 `openid` scope，
        // 这里只需补加 profile / email，避免 scope 重复。
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_str()))
}

/// `GET /api/oidc/callback`
///
/// Exchanges the authorization code for tokens, verifies the user against the
/// configured allowlist, issues a local JWT, and redirects back to the frontend
/// with `?token=<jwt>`.
pub async fn oidc_callback(
    Query(query): Query<OidcCallbackQuery>,
    State(config): State<Arc<OidcConfig>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let redirect_url = build_redirect_url(&headers)?;
    let client = build_client(&config, redirect_url).await?;

    // Recover the frontend redirect and nonce from the state payload.
    let state_raw = query.state.strip_prefix("r:").unwrap_or("").to_string();
    let (redirect_part, nonce_part) = state_raw.split_once("|n:").unwrap_or(("", ""));
    let frontend_redirect = redirect_part.to_string();
    let nonce = Nonce::new(nonce_part.to_string());

    let code = AuthorizationCode::new(query.code);
    let token_response = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await
        .map_err(|e| AppError::Unauthorized(format!("OIDC token exchange failed: {}", e)))?;

    // Extract identity from the id_token claims (sub, email, name).
    let id_token = token_response
        .id_token()
        .ok_or_else(|| AppError::Unauthorized("OIDC response missing id_token".to_string()))?;

    let claims = id_token
        .claims(&client.id_token_verifier(), &nonce)
        .map_err(|e| AppError::Unauthorized(format!("Invalid OIDC id_token: {}", e)))?;

    let sub = claims.subject().to_string();

    // Authelia 的 id_token 默认只含 sub，email/name 等 claims 需通过
    // userinfo 端点获取（OIDC 标准流程）。若 id_token 已含 email 则无需调用。
    let (email, name) = if let Some(e) = claims.email() {
        (Some(e.to_string()), None)
    } else {
        match client.user_info(token_response.access_token().to_owned(), Some(claims.subject().clone())) {
            Ok(req) => match req
                .request_async::<EmptyAdditionalClaims, _, _, CoreGenderClaim, _>(async_http_client)
                .await
            {
                Ok(ui) => (
                    ui.email().map(|e| e.to_string()),
                    ui.name()
                        .and_then(|n| n.get(None))
                        .map(|n| n.to_string()),
                ),
                Err(e) => {
                    tracing::warn!("OIDC userinfo request failed: {:?}", e);
                    (None, None)
                }
            },
            Err(e) => {
                tracing::warn!("OIDC userinfo unavailable: {:?}", e);
                (None, None)
            }
        }
    };

    // Allowlist check: email OR subject must match.
    // allow_emails 与 allow_subjects 至少配置一项（配置校验强制），不再放行任意用户。
    let email_ok = email
        .as_ref()
        .map(|e| config.allow_emails.iter().any(|a| a.eq_ignore_ascii_case(e)))
        .unwrap_or(false);
    let sub_ok = config.allow_subjects.iter().any(|s| s == &sub);

    if !email_ok && !sub_ok {
        return Err(AppError::Forbidden(
            "User is not in the OIDC allowlist".to_string(),
        ));
    }

    // Issue local JWT.
    let jwt_claims = crate::jwt::build_claims(&sub, email, name);
    let token = crate::jwt::encode(&jwt_claims)?;

    // Redirect back to the frontend with the token in the query string.
    //
    // 注意：Location 必须是「路径 + 查询串」形式（如 `/?token=...`），
    // 不能是 fragment-only 的 `#/...`。否则浏览器收到 303 后会再次请求
    // 当前 URL（即 callback 本身），导致一次性的 authorization code 被
    // 重复兑换，IdP 返回 40001「OIDC token exchange failed」。
    // 前端 SPA 使用 HashRouter，通过 window.location.search 读取 token。
    let mut location = format!("/?token={}", urlencode(&token));
    if !frontend_redirect.is_empty() {
        location.push_str("&redirect=");
        location.push_str(&urlencode(&frontend_redirect));
    }
    Ok(Redirect::to(&location))
}

fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => {
                out.push('%');
                out.push(
                    char::from_digit((b >> 4) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
                out.push(
                    char::from_digit((b & 0xf) as u32, 16)
                        .unwrap()
                        .to_ascii_uppercase(),
                );
            }
        }
    }
    out
}
