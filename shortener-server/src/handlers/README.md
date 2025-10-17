# HTTP 处理器

此模块包含 shortener-server API 的所有 HTTP 请求处理器。

## 结构

```
handlers/
├── mod.rs          # 模块导出
├── shorten.rs      # 短链接管理处理器
├── account.rs      # 认证处理器
└── history.rs      # 访问历史处理器
```

## 处理器概述

### 短链接管理（`shorten.rs`）

#### POST /api/shortens
创建新的短链接。

**请求体：**
```json
{
  "original_url": "https://example.com",
  "code": "custom",           // 可选：自定义短代码
  "describe": "描述"          // 可选：描述
}
```

#### GET /api/shortens
列出短链接（分页）。

#### GET /api/shortens/{code}
获取短链接详情。

#### PUT /api/shortens/{code}
更新短链接。

#### DELETE /api/shortens/{code}
删除短链接。

### 账户管理（`account.rs`）

#### POST /api/account/login
用户登录并获取 JWT 令牌。

#### POST /api/account/logout
用户登出。

#### GET /api/users/current
获取当前用户信息。

### 访问历史（`history.rs`）

#### GET /api/histories
列出访问历史（分页）。

#### DELETE /api/histories
批量删除历史记录。
