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
  "code": "abc123",
  "short_url": "http://localhost:8080/abc123",
  "original_url": "https://example.com",
  "describe": "示例网站",
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
    "page_size": 10,
    "current_count": 10,
    "total_items": 100,
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

### 账户管理

#### 登录

获取用于认证的 JWT 令牌。

```http
POST /api/account/login
Content-Type: application/json

{
  "username": "admin",
  "password": "your-password",
  "auto": false
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
  "code": "mylink",
  "describe": "示例网站"
}
```

参数：

- `original_url`（必需）：原始长 URL
- `code`（可选）：自定义短代码（未提供则自动生成）
- `describe`（可选）：URL 描述

示例：

```bash
curl -X POST http://localhost:8080/api/shortens \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "original_url": "https://example.com",
    "code": "mylink",
    "describe": "示例网站"
  }'
```

#### 获取短链接

获取特定短链接的详情。

```http
GET /api/shortens/{code}
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
GET /api/shortens?page=1&page_size=10&sort_by=created_at&order=desc&status=1
X-API-KEY: your-api-key
```

查询参数：

- `page`（可选，默认：1）：页码
- `page_size`（可选，默认：10）：每页项数
- `sort_by`（可选，默认：created_at）：排序字段
- `order`（可选，默认：desc）：排序顺序（asc、desc）
- `code`（可选）：按短链接代码过滤
- `original_url`（可选）：按原始URL模糊查找
- `status`（可选）：按状态过滤（0=启用，1=禁用）

示例：

```bash
# 基本查询
curl "http://localhost:8080/api/shortens?page=1&page_size=10" \
  -H "X-API-KEY: your-api-key"

# 按短链接代码过滤
curl "http://localhost:8080/api/shortens?code=gitmirror" \
  -H "X-API-KEY: your-api-key"

# 按原始URL模糊查找
curl "http://localhost:8080/api/shortens?original_url=github.com" \
  -H "X-API-KEY: your-api-key"

# 按状态过滤
curl "http://localhost:8080/api/shortens?status=0" \
  -H "X-API-KEY: your-api-key"

# 组合过滤
curl "http://localhost:8080/api/shortens?page=1&page_size=10&sort_by=created_at&order=desc&code=gitmirror&original_url=github&status=0" \
  -H "X-API-KEY: your-api-key"
```

#### 更新短链接

更新现有短链接。

```http
PUT /api/shortens/{code}
X-API-KEY: your-api-key
Content-Type: application/json

{
  "original_url": "https://newurl.com",
  "describe": "更新的描述"
}
```

示例：

```bash
curl -X PUT http://localhost:8080/api/shortens/mylink \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "original_url": "https://newurl.com",
    "describe": "更新的描述"
  }'
```

#### 删除短链接

删除特定短链接。

```http
DELETE /api/shortens/{code}
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

### 访问历史

#### 列出访问历史

获取访问历史记录的分页列表。

```http
GET /api/histories?page=1&page_size=10&sort_by=accessed_at&order=desc
X-API-KEY: your-api-key
```

查询参数：

- `page`（可选，默认：1）：页码
- `page_size`（可选，默认：10）：每页项数
- `sort_by`（可选，默认：accessed_at）：排序字段
- `order`（可选，默认：desc）：排序顺序
- `ip_address`（可选）：按IP地址过滤
- `short_code`（可选）：按短链接代码过滤
- `url_id`（可选）：按URL ID过滤

示例：

```bash
# 基本查询
curl "http://localhost:8080/api/histories?page=1&page_size=10" \
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
  -d '{"original_url":"https://example.com","code":"test"}'

# 3. 获取短链接
curl http://localhost:8080/api/shortens/test \
  -H "Authorization: Bearer $TOKEN"

# 4. 列出所有 URL
curl "http://localhost:8080/api/shortens?page=1&page_size=10" \
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
