# 配置指南

Shortener 服务器的完整配置参考。

## 目录

- [概述](#概述)
- [配置文件](#配置文件)
- [环境变量](#环境变量)
- [服务器配置](#服务器配置)
- [管理员配置](#管理员配置)
- [OIDC 配置](#oidc-配置)
- [数据库配置](#数据库配置)
- [缓存配置](#缓存配置)
- [GeoIP 配置](#geoip-配置)

## 概述

Shortener 服务器使用 TOML 格式进行配置。配置可以从以下位置加载：

1. 配置文件（默认：`config.toml`，**可选**——文件不存在时以纯环境变量启动）
2. 环境变量（`__` 分隔嵌套键（无前缀））

优先级（从高到低）：环境变量 > 配置文件 > 内置缺省值

> `--config` 命令行参数仅用于指定配置文件路径，不直接提供配置值。
> 另有三个平铺别名环境变量（`DATABASE_URL` / `CACHE_URL` / `OIDC_CLIENT_SECRET`），
> 仅当对应的 `__` 嵌套形式未设置时生效。

## 配置文件

### 位置

默认位置（按优先级顺序）：

1. `--config` 标志指定的路径
2. `./config.toml`
3. `/etc/shortener/config.toml`

### 格式

```toml
[server]
# 服务器设置

[slug]
# 短链接 slug 生成设置

[admin]
# 管理员账户设置

[database]
# 数据库连接设置

[cache]
# 缓存设置

[geoip]
# GeoIP 设置
```

## 环境变量

环境变量使用双下划线 `__` 分隔嵌套键（无前缀）：

```bash
# 服务器地址
export SERVER__ADDRESS=":9090"

# 数据库连接 URL（覆盖配置文件的 database.url）
export DATABASE__URL="postgres://user:pass@localhost:5432/shortener?sslmode=require"

# 缓存连接 URL（覆盖配置文件的 cache.url）
export CACHE__URL="redis://:password@localhost:6379/0"

# 启用缓存
export CACHE__ENABLED="true"

# API 密钥
export SERVER__API_KEY="your-secret-key"
```

> 📖 完整的变量清单、默认值与各部署方式示例，请参阅 [环境变量参考](ENVIRONMENT_VARIABLES.md)。

## 服务器配置

```toml
[server]
address = ":8080"                          # 监听地址
short_url = "https://s.example.com"      # 短址专用域名（可选，未设置时从监听地址推断，通配地址回退 localhost）
api_key = "your-secret-api-key"           # API 密钥（必需）
```

### 详细说明

- `address`：服务器监听地址，默认 `:8080`
- `short_url`：短址专用域名，用于生成短链接；未设置时从监听地址推断，通配地址回退 localhost
- `api_key`：用于认证的 API 密钥，使用 `openssl rand -base64 32` 生成

## 短链接配置

```toml
[slug]
length = 6                              # 短链接 slug 长度（4-16）
alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
```

## 管理员配置

管理员账号用于密码登录通道。口令以 **Argon2id 哈希**（PHC 字符串格式）存储，不以明文保存。

```toml
[admin]
username = "admin"                        # 管理员用户名（可选，默认 "admin"）
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$..."  # 口令 Argon2id 哈希（必需）
```

生成哈希（任选其一）：

```bash
# 服务端自带的子命令（推荐，无需额外安装 CLI）
shortener-server hash-password --password "your-secure-password"

# 或使用命令行工具
shortener-cli hash-password --password "your-secure-password"
```

交互式输入（不在 shell 历史中留痕）：

```bash
shortener-server hash-password          # 按提示输入口令
shortener-cli hash-password
```

将输出的整行（`$argon2id$...`）粘贴为 `password_hash` 的值。

> 密码登录与 OIDC 登录为**并存**的两条独立通道，详见 [OIDC 配置](#oidc-配置)。

## OIDC 配置

通过标准 OIDC / OAuth2.0（授权码流）对接任意身份提供方（IdP），如 Keycloak、Authelia、Okta、Microsoft Entra ID、Google 等。登录时仅**白名单**内的 IdP 用户可登录。

```toml
[oidc]
# OIDC 登录总开关（false 时 OIDC 登录不可用）
enabled = false

# IdP 的 issuer 地址（自动发现端点 .well-known/openid-configuration 的前缀）
# enabled 时必填
issuer = "https://keycloak.example.com/realms/shortener"

# 在 IdP 注册的 OAuth2 客户端 ID
client_id = "shortener-app"

# 客户端密钥。可留空（公钥客户端），或通过环境变量 OIDC__CLIENT_SECRET 注入
client_secret = ""

# 回调地址由服务根据请求的 Host 头自动推导（https://<域名>/api/oidc/callback），
# 无需在此配置；请确保 IdP 中登记的回调地址与该地址完全一致

# 允许登录的用户白名单（email 与/或 sub 任一命中即放行；至少配置一项，不能都为空）
allow_emails = ["admin@example.com"]
allow_subjects = []
```

### 环境变量覆盖

敏感配置优先使用环境变量（双下划线 `__` 分隔嵌套键，无前缀），`client_secret` 强烈建议通过环境变量注入，避免写入文件：

```bash
export OIDC__CLIENT_SECRET="your-client-secret"
# 等价于配置项 [oidc] client_secret
```

JWT 签名密钥（两条登录通道签发令牌都依赖它）也必须通过环境变量提供：

```bash
export JWT_SECRET="$(openssl rand -base64 48)"
```

若密钥以文件形式挂载（systemd `LoadCredential`、Docker / K8s Secret），改用 `JWT_SECRET_FILE` 指向该文件：

```bash
export JWT_SECRET_FILE=/run/secrets/jwt_secret
```

两者同时设置时以 `JWT_SECRET` 为准；文件末尾的换行会被自动忽略，文件为空或不可读则启动失败。

### 在 IdP 侧的配置要点

1. 创建 OAuth2 / OIDC 客户端（授权码流，`response_type=code`）。
2. 将回调地址（Redirect URI / Callback URL）设置为本服务的
   `https://<你的域名>/api/oidc/callback`。
3. 申请 `openid`、`profile`、`email` 三个 scope。
4. 若 IdP 要求客户端密钥，将其填入 `client_secret` 或 `OIDC__CLIENT_SECRET`。

### 登录流程

1. 访问 `GET /api/oidc/login` → 浏览器被重定向到 IdP 授权页（登录后固定跳转前端 `/#/dashboard`）。
2. 用户在 IdP 完成认证后，IdP 回调 `GET /api/oidc/callback?code=...&state=...`。
3. 服务用 `code` 换取 token，校验 `id_token` 签名，并比对 `email`/`sub` 白名单。
4. 校验通过则签发本地 JWT，302 跳回前端并附带 `?token=<jwt>`；前端将其存入 `localStorage`。

前端登录页已内置「使用 OIDC 登录」按钮，点击即触发上述流程。

### 多实例部署

JWT 采用无状态 HS256 签名，只要所有实例使用**同一个 `JWT_SECRET`**，即可共享校验、支持横向扩展。

## 数据库配置

数据库连接通过单个 URL 配置，引擎类型（sqlite / postgres / postgresql / mysql）由 URL 的 scheme 自动推断。

```toml
[database]
url = "sqlite://data/shortener.db?mode=rwc"  # 连接 URL（必需）
log_level = 1
```

不同引擎的 `url` 示例：

- SQLite（文件）：`sqlite://data/shortener.db?mode=rwc`
- SQLite（内存，仅测试）：`sqlite::memory:`
- PostgreSQL：`postgres://user:pass@localhost:5432/shortener?sslmode=disable`
- MySQL：`mysql://user:pass@localhost:3306/shortener?charset=utf8mb4`

## 缓存配置

缓存连接同样通过 URL 配置，引擎（redis / valkey）由 scheme 推断。

```toml
[cache]
enabled = true
url = "redis://:password@localhost:6379/0"   # 连接 URL
expire = 3600
prefix = "shorten:"
```

缓存 URL 示例：

- Redis：`redis://:password@localhost:6379/0`
- Valkey：`valkey://:password@localhost:6379/0`

## GeoIP 配置

GeoIP 功能用于追踪访问者的地理位置信息。默认禁用，需要手动配置。

```toml
[geoip]
enabled = false  # 默认禁用
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
```

### 启用 GeoIP

要启用 GeoIP 功能，需要：

1. 下载 ip2region 数据库文件：
   ```bash
   curl -fsSL https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb \
       -o data/ip2region.xdb
   ```

2. 在配置文件中启用：
   ```toml
   [geoip]
   enabled = true
   ```

3. 重启服务

详细的 GeoIP 配置和使用说明，请参阅 [GeoIP 配置指南](GEOIP.md)。

## 配置示例

### 开发环境

```toml
[server]
address = ":8080"
short_url = "http://localhost:8080"
api_key = "dev-api-key"

[admin]
username = "admin"
# 生成：shortener-server hash-password "admin123"
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$devonly$devonlydevonlydevonlydevonly"

[database]
url = "sqlite://data/shortener.db?mode=rwc"

[cache]
enabled = false

[geoip]
enabled = false
```

### 生产环境

```toml
[server]
address = ":8080"
short_url = "https://short.example.com"
api_key = "${SHORTENER_API_KEY}"

[admin]
username = "${SHORTENER_ADMIN_USER}"
# 生成：shortener-server hash-password "${SHORTENER_ADMIN_PASS}"
password_hash = "${SHORTENER_ADMIN_PASS_HASH}"

[database]
url = "postgres://shortener:${POSTGRES_PASSWORD}@postgres:5432/shortener?sslmode=require"

[cache]
enabled = true
url = "redis://:${REDIS_PASSWORD}@redis:6379/0"

[geoip]
enabled = true

[geoip.ip2region]
path = "/var/lib/shortener/ip2region.xdb"
```

## 最佳实践

1. 使用环境变量存储敏感数据（密码、API 密钥）
2. 不同环境使用不同配置（开发、生产）
3. 定期轮换 API 密钥和密码
4. 生产环境使用适当的日志级别
5. 启用缓存以提高性能

## 另见

- [GeoIP 配置指南](GEOIP.md)
- [API 文档](../server/API.md)
- [部署指南](../deployment/DEPLOYMENT.md)
- [Docker 部署](../deployment/DOCKER.md)
