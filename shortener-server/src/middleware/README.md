# 中间件模块

此模块为 shortener-server 应用程序提供三个基本的中间件组件。

## 组件

### 1. API 密钥认证（`api_key_auth`）

从 `X-API-KEY` 头验证 API 密钥。

**使用：**
```rust
use shortener_server::middleware::ApiKeyAuth;
use axum::middleware;

let api_key_auth = ApiKeyAuth::new(config.server.api_key.clone());

let protected_routes = Router::new()
    .route("/api/shortens", get(list_shortens))
    .layer(middleware::from_fn(move |headers, req, next| {
        ApiKeyAuth::check_api_key(api_key_auth.api_key.clone(), headers, req, next)
    }));
```

**特性：**
- 根据配置的 API 密钥验证 `X-API-KEY` 头
- 对于缺失或无效的密钥返回 401 Unauthorized
- 记录认证尝试以进行安全监控

### 2. 日志中间件（`logging`）

记录所有 HTTP 请求和响应。

**特性：**
- 记录请求方法、路径、状态码
- 记录请求处理时间
- 记录客户端 IP 地址
- 记录用户代理

### 3. JWT 认证（`jwt_auth`）

验证 JWT 令牌以进行用户认证。

**特性：**
- 从 `Authorization` 头提取 JWT 令牌
- 验证令牌签名和过期时间
- 将用户信息注入请求扩展
- 对于无效令牌返回 401 Unauthorized
