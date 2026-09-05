# API 文档

Shortener 服务器的完整 API 参考。

## 基础 URL

```
http://localhost:8080
```

## 认证

所有 API 端点（健康检查除外）需要使用以下方式之一进行认证：

### API 密钥认证

在请求头中包含 API 密钥：

```
X-API-KEY: your-api-key
```

### JWT 令牌认证

1. 登录获取令牌
2. 在后续请求中包含令牌：

```
Authorization: Bearer <your-jwt-token>
```

## 响应格式

### 成功响应

```json
{
  "id": 1,
  "short_code": "abc123",
  "short_url": "http://localhost:8080/abc123",
  "original_url": "https://example.com",
  "description": "示例网站",
  "status": 1,
  "created_at": "2024-03-20T12:00:00Z",
  "updated_at": "2024-03-20T12:00:00Z"
}
```

### 错误响应

```json
{
  "errcode": "NOT_FOUND",
  "errinfo": "未找到短链接"
}
```

### 分页响应

```json
{
  "data": [...],
  "meta": {
    "page": 1,
    "per_page": 10,
    "count": 10,
    "total": 100,
    "total_pages": 10
  }
}
```

## HTTP 状态码

- `200 OK` - 请求成功
- `201 Created` - 资源创建成功
- `204 No Content` - 请求成功，无内容返回
- `400 Bad Request` - 无效的请求参数
- `401 Unauthorized` - 需要认证或认证失败
- `403 Forbidden` - 权限不足
- `404 Not Found` - 资源未找到
- `409 Conflict` - 资源已存在
- `500 Internal Server Error` - 服务器错误

## 端点

### 健康检查

#### 健康检查 / 服务信息

无需认证。

```http
GET /ping
```

响应：

```json
{
  "message": "pong"
}
```

```http
GET /
```

返回服务信息（名称、版本、状态）：

```json
{
  "service": "URL Shortener API",
  "version": "0.2.0-preview.1",
  "status": "running"
}
```

示例：

```bash
curl http://localhost:8080/ping
```

### 账户管理

#### 登录

获取用于认证的 JWT 令牌。

```http
POST /api/account/login
Content-Type: application/json

{
  "username": "admin",
  "password": "your-password",
  "auto_login": false
}
```

响应：

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

示例：

```bash
curl -X POST http://localhost:8080/api/account/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
```

#### 登出

使当前 JWT 令牌失效。

```http
POST /api/account/logout
Authorization: Bearer <token>
```

#### 获取当前用户

获取当前认证用户的信息。

```http
GET /api/users/current
Authorization: Bearer <token>
```

### 短链接管理

#### 创建短链接

创建新的短链接。

```http
POST /api/shortens
X-API-KEY: your-api-key
Content-Type: application/json

{
  "original_url": "https://example.com",
  "short_code": "mylink",
  "description": "示例网站"
}
```

参数：

- `original_url`（必需）：原始长 URL
- `short_code`（可选）：自定义短代码（未提供则自动生成）
- `description`（可选）：URL 描述

示例：

```bash
curl -X POST http://localhost:8080/api/shortens \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "original_url": "https://example.com",
    "short_code": "mylink",
    "description": "示例网站"
  }'
```

#### 获取短链接

获取特定短链接的详情。路径参数名为 `short_code`。

```http
GET /api/shortens/{short_code}
X-API-KEY: your-api-key
```

示例：

```bash
curl http://localhost:8080/api/shortens/mylink \
  -H "X-API-KEY: your-api-key"
```

#### 列出短链接

获取短链接的分页列表。

```http
GET /api/shortens?page=1&per_page=10&sort_by=created_at&order=desc&status=1
X-API-KEY: your-api-key
```

查询参数：

- `page`（可选，默认：1）：页码
- `per_page`（可选，默认：10）：每页项数
- `sort_by`（可选，默认：created_at）：排序字段
- `order`（可选，默认：desc）：排序顺序（asc、desc）
- `short_code`（可选）：按短链接代码过滤
- `original_url`（可选）：按原始URL模糊查找
- `status`（可选）：按状态过滤（0=启用，1=禁用）

示例：

```bash
# 基本查询
curl "http://localhost:8080/api/shortens?page=1&per_page=10" \
  -H "X-API-KEY: your-api-key"

# 按短链接代码过滤
curl "http://localhost:8080/api/shortens?short_code=gitmirror" \
  -H "X-API-KEY: your-api-key"

# 按原始URL模糊查找
curl "http://localhost:8080/api/shortens?original_url=github.com" \
  -H "X-API-KEY: your-api-key"

# 按状态过滤
curl "http://localhost:8080/api/shortens?status=0" \
  -H "X-API-KEY: your-api-key"

# 组合过滤
curl "http://localhost:8080/api/shortens?page=1&per_page=10&sort_by=created_at&order=desc&short_code=gitmirror&original_url=github&status=0" \
  -H "X-API-KEY: your-api-key"
```

#### 更新短链接

更新现有短链接。路径参数名为 `short_code`。

```http
PUT /api/shortens/{short_code}
X-API-KEY: your-api-key
Content-Type: application/json

{
  "original_url": "https://newurl.com",
  "description": "更新的描述"
}
```

示例：

```bash
curl -X PUT http://localhost:8080/api/shortens/mylink \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "original_url": "https://newurl.com",
    "description": "更新的描述"
  }'
```

#### 删除短链接

删除特定短链接。路径参数名为 `short_code`。

```http
DELETE /api/shortens/{short_code}
X-API-KEY: your-api-key
```

示例：

```bash
curl -X DELETE http://localhost:8080/api/shortens/mylink \
  -H "X-API-KEY: your-api-key"
```

#### 批量删除短链接

一次删除多个短链接。

```http
POST /api/shortens/batch-delete
X-API-KEY: your-api-key
Content-Type: application/json

{
  "ids": [1, 2, 3]
}
```

示例：

```bash
curl -X POST "http://localhost:8080/api/shortens/batch-delete" \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"ids": [1, 2, 3]}'
```

### OIDC 登录

通过标准 OIDC / OAuth2.0 授权码流对接外部身份提供方（IdP）进行单点登录。

#### 发起登录

将浏览器重定向到 IdP 授权页。登录成功后固定跳转前端 `/#/dashboard`。

```http
GET /api/oidc/login
```

成功时返回 `303 See Other`，`Location` 指向 IdP 授权端点。

> 回调地址无需配置：服务根据请求 `Host` 头自动推导为
> `https://<域名>/api/oidc/callback`，请确保 IdP 登记的 Redirect URI 与之完全一致。

#### 回调

IdP 认证完成后回调本服务：

```http
GET /api/oidc/callback?code=...&state=...
```

校验通过后签发 JWT，`302` 跳回前端并附带 `?token=<jwt>`；前端将 token 存入 `localStorage`。

> email / name 若 id_token 缺失（如 Authelia），服务自动调用 userinfo 端点补充获取后再比对白名单。

> 详细对接配置请参阅 [OIDC 对接部署](../general/OIDC.md)。

### 访问历史

#### 列出访问历史

获取访问历史记录的分页列表。

```http
GET /api/histories?page=1&per_page=10&sort_by=accessed_at&order=desc
X-API-KEY: your-api-key
```

查询参数：

- `page`（可选，默认：1）：页码
- `per_page`（可选，默认：10）：每页项数
- `sort_by`（可选，默认：accessed_at）：排序字段
- `order`（可选，默认：desc）：排序顺序
- `ip_address`（可选）：按IP地址过滤
- `short_code`（可选）：按短链接代码过滤
- `url_id`（可选）：按URL ID过滤

示例：

```bash
# 基本查询
curl "http://localhost:8080/api/histories?page=1&per_page=10" \
  -H "X-API-KEY: your-api-key"

# 按IP地址过滤
curl "http://localhost:8080/api/histories?ip_address=192.168.1.1" \
  -H "X-API-KEY: your-api-key"

# 按短链接代码过滤
curl "http://localhost:8080/api/histories?short_code=abc123" \
  -H "X-API-KEY: your-api-key"
```

#### 批量删除历史

一次删除多个历史记录。

```http
POST /api/histories/batch-delete
X-API-KEY: your-api-key
Content-Type: application/json

{
  "ids": [1, 2, 3]
}
```

示例：

```bash
curl -X POST "http://localhost:8080/api/histories/batch-delete" \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"ids": [1, 2, 3]}'
```

### 缓存管理

#### 刷新缓存

清空缓存中本服务前缀（默认 `shorten:`）下的所有键，然后从数据库重新加载全部短链到缓存。适用于缓存与数据库出现不一致、或需要立即生效的场景。

支持 API 密钥或 JWT 令牌认证（与短链接管理接口一致）。

```http
POST /api/cache/refresh
X-API-KEY: your-api-key
```

成功响应（200）：

```json
{
  "cleared_keys": 3,
  "warmed_urls": 3
}
```

- `cleared_keys`：清除的旧缓存键数量
- `warmed_urls`：从数据库重新缓存的短链数量

说明：

- 仅清除**本服务前缀**（`[cache] prefix`，默认 `shorten:`）开头的键，不影响同一 Valkey 实例中其他应用的键
- 缓存未启用（`cache.enabled = false`）或连接失败时返回缓存错误

示例：

```bash
curl -X POST "http://localhost:8080/api/cache/refresh" \
  -H "X-API-KEY: your-api-key"
```

## 错误代码

| 代码 | 描述 |
|------|------|
| `SYSTEM_ERROR` | 内部系统错误 |
| `CONFIG_ERROR` | 配置错误 |
| `URL_NOT_FOUND` | 未找到短链接 |
| `CODE_EXISTS` | 短代码已存在 |
| `INVALID_URL` | 无效的 URL 格式 |
| `UNAUTHORIZED` | 需要认证或认证失败 |
| `FORBIDDEN` | 权限不足 |
| `NOT_FOUND` | 资源未找到 |
| `BAD_REQUEST` | 无效的请求参数 |
| `DATABASE_ERROR` | 数据库操作失败 |
| `CACHE_ERROR` | 缓存操作失败 |
| `GEOIP_ERROR` | GeoIP 查找失败 |

## 完整工作流示例

```bash
# 1. 登录
TOKEN=$(curl -s -X POST http://localhost:8080/api/account/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password"}' \
  | jq -r '.token')

# 2. 创建短链接
curl -X POST http://localhost:8080/api/shortens \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"original_url":"https://example.com","short_code":"test"}'

# 3. 获取短链接
curl http://localhost:8080/api/shortens/test \
  -H "Authorization: Bearer $TOKEN"

# 4. 列出所有 URL
curl "http://localhost:8080/api/shortens?page=1&per_page=10" \
  -H "Authorization: Bearer $TOKEN"

# 5. 更新 URL
curl -X PUT http://localhost:8080/api/shortens/test \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"original_url":"https://newurl.com"}'

# 6. 删除 URL
curl -X DELETE http://localhost:8080/api/shortens/test \
  -H "Authorization: Bearer $TOKEN"

# 7. 登出
curl -X POST http://localhost:8080/api/account/logout \
  -H "Authorization: Bearer $TOKEN"
```

## OpenAPI 规范

完整的机器可读 API 规范，请参阅 [openapi.yml](https://github.com/jetsung/shortener/blob/main/openapi.yml)。

你可以使用此规范配合以下工具：
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Postman](https://www.postman.com/)
- [Insomnia](https://insomnia.rest/)

## 支持

API 支持：
- 🐛 [报告问题](https://github.com/jetsung/shortener/issues)
