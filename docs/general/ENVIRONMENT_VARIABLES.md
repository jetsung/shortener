# 环境变量参考

Shortener 服务器的所有配置都可以通过环境变量覆盖。本文档是**完整的环境变量参数参考**，涵盖命名规则、每个变量的用法、默认值与示例。

## 目录

- [加载优先级](#加载优先级)
  - [最小必填项](#最小必填项)
- [命名规则](#命名规则)
- [完整速查表](#完整速查表)
- [服务器配置](#服务器配置-server)
- [短链接配置](#短链接配置-slug)
- [管理员配置](#管理员配置-admin)
- [OIDC 配置](#oidc-配置-oidc)
- [数据库配置](#数据库配置-database)
- [缓存配置](#缓存配置-cache)
- [GeoIP 配置](#geoip-配置-geoip)
- [日志配置](#日志配置-logging)
- [JWT 密钥](#jwt-密钥)
- [其他环境变量](#其他环境变量)
- [使用示例](#使用示例)
- [常见问题](#常见问题)

## 加载优先级

配置来源（从高到低，后者仅在高位未提供时生效）：

1. **环境变量**（`__` 嵌套形式，如 `SERVER__ADDRESS`）
2. **平铺别名**（仅 `DATABASE_URL` / `CACHE_URL` / `OIDC_CLIENT_SECRET` 三个，且仅当对应 `__` 形式未设置时生效）
3. **配置文件** `config.toml`（默认路径 `./config.toml`，可用 `--config` 指定其他路径）
4. **内置缺省值**（serde default + `apply_defaults()`）

```text
环境变量（__ 形式） > 平铺别名 > 配置文件 > 缺省值
```

> **`config.toml` 是可选的**：文件不存在时服务直接以纯环境变量（加缺省值）启动，
> 日志会提示 `Configuration file 'config.toml' not found, using environment variables only`。
> `--config` 只是配置文件的路径参数，不参与上述优先级。

### 最小必填项

无论使用文件还是纯环境变量，以下配置必须提供（缺失时启动报字段级错误）：

| 环境变量 | 说明 |
| --- | --- |
| `JWT_SECRET`（或 `JWT_SECRET_FILE`） | JWT 签名密钥，独立于配置文件，见 [JWT 密钥](#jwt-密钥) |
| `SERVER__API_KEY` | API 认证密钥 |
| `ADMIN__PASSWORD_HASH` | 管理员口令哈希 |
| `DATABASE__URL` | 数据库连接串 |

其余全部配置项均有合理默认值（如监听地址 `:8080`、管理员用户名 `admin`、短码长度 `6`）。

## 命名规则

环境变量使用**双下划线 `__`** 分隔嵌套配置键，**无前缀**：

```bash
# TOML 配置             环境变量
[server] address    →   SERVER__ADDRESS
[oidc] issuer       →   OIDC__ISSUER
[geoip.ip2region]   →   GEOIP__IP2REGION__PATH
```

- 布尔值使用字符串 `"true"` / `"false"`
- 列表值支持 `"[]"`、`["a","b"]`（JSON 风格）、`a,b`（逗号分隔）三种写法

## 完整速查表

| 环境变量 | 对应配置键 | 必需 | 默认值 |
| --- | --- | --- | --- |
| `SERVER__ADDRESS` | `[server] address` | 否 | `:8080` |
| `SERVER__TRUSTED_PLATFORM` | `[server] trusted-platform` | 否 | 空 |
| `SERVER__SHORT_URL` | `[server] short_url` | 否 | 空（从监听地址推断） |
| `SERVER__API_KEY` | `[server] api_key` | **是** | - |
| `SLUG__LENGTH` | `[slug] length` | 否 | `6` |
| `SLUG__ALPHABET` | `[slug] alphabet` | 否 | 大小写字母+数字 |
| `ADMIN__USERNAME` | `[admin] username` | 否 | `admin` |
| `ADMIN__PASSWORD_HASH` | `[admin] password_hash` | **是** | - |
| `OIDC__ENABLED` | `[oidc] enabled` | 否 | `false` |
| `OIDC__ISSUER` | `[oidc] issuer` | 启用时必填 | - |
| `OIDC__CLIENT_ID` | `[oidc] client_id` | 启用时必填 | - |
| `OIDC__CLIENT_SECRET` | `[oidc] client_secret` | 否 | - |
| `OIDC__ALLOW_EMAILS` | `[oidc] allow_emails` | 启用时至少一项非空 | `[]` |
| `OIDC__ALLOW_SUBJECTS` | `[oidc] allow_subjects` | 启用时至少一项非空 | `[]` |
| `DATABASE__URL` | `[database] url` | **是** | - |
| `DATABASE__LOG_LEVEL` | `[database] log_level` | 否 | `1` |
| `CACHE__ENABLED` | `[cache] enabled` | 否 | `false` |
| `CACHE__URL` | `[cache] url` | 启用时必填 | - |
| `CACHE__EXPIRE` | `[cache] expire` | 否 | `3600` |
| `CACHE__PREFIX` | `[cache] prefix` | 否 | `shorten:` |
| `GEOIP__ENABLED` | `[geoip] enabled` | 否 | `false` |
| `GEOIP__TYPE` | `[geoip] type` | 否 | `ip2region` |
| `GEOIP__IP2REGION__PATH` | `[geoip.ip2region] path` | 启用时必填 | - |
| `GEOIP__IP2REGION__MODE` | `[geoip.ip2region] mode` | 否 | `vector` |
| `GEOIP__IP2REGION__VERSION` | `[geoip.ip2region] version` | 否 | `4` |
| `LOGGING__LEVEL` | `[logging] level` | 否 | `info` |
| `LOGGING__FORMAT` | `[logging] format` | 否 | `pretty` |
| `LOGGING__WITH_TIMESTAMP` | `[logging] with_timestamp` | 否 | `true` |
| `LOGGING__WITH_TARGET` | `[logging] with_target` | 否 | `true` |
| `LOGGING__WITH_THREAD_IDS` | `[logging] with_thread_ids` | 否 | `false` |
| `LOGGING__WITH_THREAD_NAMES` | `[logging] with_thread_names` | 否 | `false` |
| `LOGGING__WITH_FILE` | `[logging] with_file` | 否 | `false` |
| `LOGGING__WITH_LINE_NUMBER` | `[logging] with_line_number` | 否 | `false` |
| `LOGGING__WITH_ANSI` | `[logging] with_ansi` | 否 | `true` |
| `JWT_SECRET` | -（独立变量） | 二选一 | - |
| `JWT_SECRET_FILE` | -（独立变量） | 二选一 | - |

> 特殊变量 `JWT_SECRET` / `JWT_SECRET_FILE` 不在 `config.toml` 中，属于独立环境变量，详见 [JWT 密钥](#jwt-密钥)。

## 服务器配置 `[server]`

| 环境变量 | 对应配置键 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `SERVER__ADDRESS` | `address` | 字符串 | `:8080` | 监听地址。Docker/生产环境使用 `0.0.0.0:8080` 接受外部连接 |
| `SERVER__TRUSTED_PLATFORM` | `trusted-platform` | 字符串 | 空 | 可信平台头（用于获取真实客户端 IP），可选 |
| `SERVER__SHORT_URL` | `short_url` | 字符串 | 空（从监听地址推断） | 短址专用域名，用于生成短链接 |
| `SERVER__API_KEY` | `api_key` | 字符串 | - | API 认证密钥，**必需**。生成：`openssl rand -base64 32` |

```bash
export SERVER__ADDRESS="0.0.0.0:8080"
export SERVER__TRUSTED_PLATFORM=""                    # 例如 "X-Real-IP"
export SERVER__SHORT_URL="https://s.example.com"  # 短址专用域名（可选，未设置时从监听地址推断，通配地址回退 localhost）
export SERVER__API_KEY="$(openssl rand -base64 32)"
```

## 短链接配置 `[slug]`

| 环境变量 | 对应配置键 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `SLUG__LENGTH` | `length` | 整数 | `6` | 短链接 slug 长度，取值范围 4–16 |
| `SLUG__ALPHABET` | `alphabet` | 字符串 | `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ` | 生成 slug 使用的字符集 |

```bash
export SLUG__LENGTH="8"
export SLUG__ALPHABET="0123456789abcdefghijklmnopqrstuvwxyz"
```

## 管理员配置 `[admin]`

管理员账号用于密码登录通道，口令以 **Argon2id 哈希**（PHC 字符串格式）存储。

| 环境变量 | 对应配置键 | 类型 | 必需 | 说明 |
| --- | --- | --- | --- | --- |
| `ADMIN__USERNAME` | `username` | 字符串 | 否（默认 `admin`） | 管理员用户名 |
| `ADMIN__PASSWORD_HASH` | `password_hash` | 字符串 | **是** | 口令的 Argon2id 哈希，生成：`shortener-server hash-password` |

```bash
export ADMIN__USERNAME="admin"
export ADMIN__PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$...'
```

> 密码登录与 OIDC 登录为**并存**的两条独立通道。口令哈希生成方式详见 [配置指南](CONFIGURATION.md#管理员配置)。

## OIDC 配置 `[oidc]`

对接任意标准 OIDC / OAuth2.0 身份提供方（IdP），仅白名单内的 IdP 用户可登录。

| 环境变量 | 对应配置键 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `OIDC__ENABLED` | `enabled` | 布尔 | `false` | OIDC 登录总开关。`false` 时 OIDC 登录不可用 |
| `OIDC__ISSUER` | `issuer` | 字符串 | - | IdP 的 issuer 地址（自动发现端点前缀），启用时必填 |
| `OIDC__CLIENT_ID` | `client_id` | 字符串 | - | 在 IdP 注册的 OAuth2 客户端 ID，启用时必填 |
| `OIDC__CLIENT_SECRET` | `client_secret` | 字符串 | - | 客户端密钥，可留空（公钥客户端） |
| `OIDC__ALLOW_EMAILS` | `allow_emails` | 列表 | `[]` | 允许登录的邮箱白名单；`enabled` 时须与 `allow_subjects` 至少一项非空 |
| `OIDC__ALLOW_SUBJECTS` | `allow_subjects` | 列表 | `[]` | 允许登录的 sub 白名单；`enabled` 时须与 `allow_emails` 至少一项非空 |

> 回调地址（`redirect_uri`）已不再作为配置项：服务根据请求 `Host` 头自动推导为
> `https://<域名>/api/oidc/callback`，仅需确保 IdP 登记的 Redirect URI 与之完全一致。

`client_secret` 敏感，强烈建议通过环境变量注入而不是写入配置文件：

```bash
export OIDC__ENABLED="true"
export OIDC__ISSUER="https://keycloak.example.com/realms/shortener"
export OIDC__CLIENT_ID="shortener-app"
export OIDC__CLIENT_SECRET="your-client-secret"
export OIDC__ALLOW_EMAILS="admin@example.com"         # 至少配置一项白名单
```

### 列表变量写法

`OIDC__ALLOW_EMAILS` / `OIDC__ALLOW_SUBJECTS` 支持三种写法：

```bash
# 1) JSON 数组风格
export OIDC__ALLOW_EMAILS='["admin@example.com","ops@example.com"]'

# 2) 逗号分隔（推荐，简洁）
export OIDC__ALLOW_EMAILS="admin@example.com,ops@example.com"

# 3) 空列表（enabled 时不允许两项都为空）
export OIDC__ALLOW_EMAILS="[]"
export OIDC__ALLOW_SUBJECTS=""
```

> 邮箱与 sub 任一命中即放行；`enabled` 时两项白名单至少配置一项，不能都为空。

## 数据库配置 `[database]`

连接通过单个 URL 配置，引擎类型（sqlite / postgres / postgresql / mysql）由 URL 的 scheme 自动推断。

| 环境变量 | 对应配置键 | 类型 | 必需 | 说明 |
| --- | --- | --- | --- | --- |
| `DATABASE__URL` | `url` | 字符串 | 是 | 数据库连接串 |
| `DATABASE__LOG_LEVEL` | `log_level` | 整数 | 否（默认 `1`） | 1=静默, 2=错误, 3=警告, 4=信息 |

```bash
# SQLite（文件）
export DATABASE__URL="sqlite://data/shortener.db?mode=rwc"

# SQLite（内存，仅测试）
export DATABASE__URL="sqlite::memory:"

# PostgreSQL
export DATABASE__URL="postgres://shortener:pass@postgres:5432/shortener?sslmode=disable"

# MySQL
export DATABASE__URL="mysql://shortener:pass@mysql:3306/shortener?charset=utf8mb4"

export DATABASE__LOG_LEVEL="1"
```

## 缓存配置 `[cache]`

缓存连接同样通过 URL 配置，引擎（redis / valkey）由 scheme 推断。

| 环境变量 | 对应配置键 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `CACHE__ENABLED` | `enabled` | 布尔 | `false` | 是否启用缓存（生产环境建议开启） |
| `CACHE__URL` | `url` | 字符串 | - | 缓存连接串，启用时必填 |
| `CACHE__EXPIRE` | `expire` | 整数 | `3600` | 缓存过期时间（秒） |
| `CACHE__PREFIX` | `prefix` | 字符串 | `shorten:` | 缓存键前缀 |

```bash
export CACHE__ENABLED="true"
export CACHE__URL="redis://:password@localhost:6379/0"   # 或 valkey://...
export CACHE__EXPIRE="3600"
export CACHE__PREFIX="shorten:"
```

缓存 URL 示例：

- Redis：`redis://:password@localhost:6379/0`
- Valkey：`valkey://:password@localhost:6379/0`

## GeoIP 配置 `[geoip]`

追踪访客地理位置，默认禁用。启用时需要 ip2region 数据库文件。

| 环境变量 | 对应配置键 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `GEOIP__ENABLED` | `enabled` | 布尔 | `false` | 是否启用 GeoIP |
| `GEOIP__TYPE` | `type` | 字符串 | `ip2region` | 提供方类型（当前仅支持 `ip2region`） |
| `GEOIP__IP2REGION__PATH` | `ip2region.path` | 字符串 | - | ip2region.xdb 数据库文件路径 |
| `GEOIP__IP2REGION__MODE` | `ip2region.mode` | 字符串 | `vector` | 搜索模式：`vector`（最快）/ `btree`（均衡）/ `binary`（最小内存） |
| `GEOIP__IP2REGION__VERSION` | `ip2region.version` | 字符串 | `4` | IP 版本：`4`（IPv4）/ `6`（IPv6） |

```bash
export GEOIP__ENABLED="true"
export GEOIP__TYPE="ip2region"
export GEOIP__IP2REGION__PATH="/var/lib/shortener/ip2region.xdb"
export GEOIP__IP2REGION__MODE="vector"
export GEOIP__IP2REGION__VERSION="4"
```

> 数据库文件下载与启用步骤详见 [GeoIP 配置指南](GEOIP.md)。

## 日志配置 `[logging]`

| 环境变量 | 对应配置键 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `LOGGING__LEVEL` | `level` | 字符串 | `info` | 日志级别：`error` / `warn` / `info` / `debug` / `trace` |
| `LOGGING__FORMAT` | `format` | 字符串 | `pretty` | 格式：`json` / `pretty` / `compact` |
| `LOGGING__WITH_TIMESTAMP` | `with_timestamp` | 布尔 | `true` | 是否包含时间戳 |
| `LOGGING__WITH_TARGET` | `with_target` | 布尔 | `true` | 是否包含模块名（target） |
| `LOGGING__WITH_THREAD_IDS` | `with_thread_ids` | 布尔 | `false` | 是否包含线程 ID |
| `LOGGING__WITH_THREAD_NAMES` | `with_thread_names` | 布尔 | `false` | 是否包含线程名 |
| `LOGGING__WITH_FILE` | `with_file` | 布尔 | `false` | 是否包含源文件名 |
| `LOGGING__WITH_LINE_NUMBER` | `with_line_number` | 布尔 | `false` | 是否包含行号 |
| `LOGGING__WITH_ANSI` | `with_ansi` | 布尔 | `true` | 是否使用 ANSI 颜色（写入日志文件时建议关闭） |

```bash
# 结构化 JSON 日志（推荐生产环境）
export LOGGING__LEVEL="info"
export LOGGING__FORMAT="json"
export LOGGING__WITH_TIMESTAMP="true"
export LOGGING__WITH_TARGET="true"
export LOGGING__WITH_THREAD_IDS="false"
export LOGGING__WITH_THREAD_NAMES="false"
export LOGGING__WITH_FILE="false"
export LOGGING__WITH_LINE_NUMBER="false"
export LOGGING__WITH_ANSI="false"
```

## JWT 密钥

JWT 用于两条登录通道（密码登录与 OIDC 登录）的令牌签发与校验，**服务启动时必须提供**。这两个变量**不在 `config.toml` 中**，只能通过环境变量设置。

| 环境变量 | 类型 | 说明 |
| --- | --- | --- |
| `JWT_SECRET` | 字符串 | JWT 签名密钥，生成：`openssl rand -base64 48` |
| `JWT_SECRET_FILE` | 字符串 | 密钥文件路径（systemd `LoadCredential`、Docker/K8s Secret 挂载时使用） |

- `JWT_SECRET` 与 `JWT_SECRET_FILE` **二选一**，两者同时设置时以 `JWT_SECRET` 为准
- 文件末尾的换行会被自动忽略；文件为空或不可读则启动失败
- 多实例部署时所有实例必须使用**同一个**密钥才能共享校验（HS256 无状态签名）

```bash
# 方式一：直接注入密钥（推荐）
export JWT_SECRET="$(openssl rand -base64 48)"

# 方式二：密钥以文件形式挂载
export JWT_SECRET_FILE=/run/secrets/jwt_secret
```

## 其他环境变量

| 环境变量 | 说明 |
| --- | --- |
| `RUST_LOG` | 可选。设置后**优先于** `LOGGING__LEVEL`，用于细粒度过滤（如 `shortener_server=info,sqlx=off`） |

```bash
export RUST_LOG="shortener_server=info,sqlx=off"
```

## 使用示例

### 纯环境变量启动（无 config.toml）

不需要配置文件，在任意目录下提供必填项即可启动：

```bash
export SERVER__API_KEY="$(openssl rand -base64 32)"
export ADMIN__PASSWORD_HASH='$(shortener-server hash-password --password "your-password")'
export DATABASE__URL="sqlite:///var/lib/shortener/shortener.db?mode=rwc"
export JWT_SECRET="$(openssl rand -base64 48)"

shortener-server   # 日志提示 "using environment variables only"
```

### Shell 直接导出

```bash
export SERVER__API_KEY="$(openssl rand -base64 32)"
export ADMIN__PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$...'
export JWT_SECRET="$(openssl rand -base64 48)"
cargo run -p shortener-server
```

### direnv / .envrc（参考仓库根目录 `.envrc`）

```bash
# .envrc
export SERVER__ADDRESS="0.0.0.0:8080"
export SERVER__SHORT_URL="https://s.example.com"
export SERVER__API_KEY="your-api-key"

export DATABASE__URL="postgres://shortener:pass@localhost:5432/shortener"
export CACHE__ENABLED="true"
export CACHE__URL="redis://:pass@localhost:6379/0"

export GEOIP__ENABLED="true"
export GEOIP__IP2REGION__PATH="data/ip2region.xdb"

export LOGGING__LEVEL="info"
export LOGGING__FORMAT="json"

export JWT_SECRET="your-jwt-secret"
```

### systemd（参考 `deploy/systemd/shortener-server.service`）

```ini
[Service]
Environment=SERVER__ADDRESS=0.0.0.0:8080
Environment=SERVER__SHORT_URL=https://s.example.com
Environment=SERVER__API_KEY=your-api-key
Environment=DATABASE__URL=postgres://shortener:pass@localhost:5432/shortener
Environment=CACHE__ENABLED=true
Environment=CACHE__URL=redis://:pass@localhost:6379/0
Environment=JWT_SECRET=your-jwt-secret
```

密钥以文件方式挂载：

```ini
[Service]
LoadCredential=jwt_secret:/etc/shortener/jwt_secret
Environment=JWT_SECRET_FILE=/run/credentials/shortener-server.service/jwt_secret
```

### Docker Compose（参考 `docker/docker-compose.yml`）

```yaml
services:
  shortener:
    image: jetsung/shortener-server:latest
    environment:
      - RUST_LOG=shortener_server=info,sqlx=off
      - DATABASE__URL=postgres://shortener:shortener_password@postgres:5432/shortener
      - CACHE__ENABLED=true
      - CACHE__URL=redis://:redis_password@redis:6379/0
      - GEOIP__ENABLED=true
      - GEOIP__IP2REGION__PATH=/app/data/ip2region.xdb
      - JWT_SECRET=change-me-in-production
```

## 常见问题

### 环境变量为什么不生效？

- 检查变量名是否使用 `__`（双下划线）分隔嵌套键，例如 `GEOIP__IP2REGION__PATH`（两层嵌套）
- 检查是否拼写为扁平别名（如 `DATABASE_URL`），扁平别名仅在 `__` 形式**未设置**时才生效
- 确认环境变量在**启动进程之前**已导出（`direnv allow` / `systemctl daemon-reload && systemctl restart`）

### 布尔值怎么写？

使用字符串 `"true"` / `"false"`（区分大小写），例如 `export CACHE__ENABLED="true"`。

### 列表值怎么写？

`OIDC__ALLOW_EMAILS` 等列表变量支持 `[]`、`["a","b"]`、`a,b` 三种写法，见 [列表变量写法](#列表变量写法)。

### 启动报错 `JWT_SECRET or JWT_SECRET_FILE environment variable is not set`？

设置 `JWT_SECRET`（或 `JWT_SECRET_FILE`）后重启服务，见 [JWT 密钥](#jwt-密钥)。

### 启动报错 `database.url is required` / `server.api_key is required`？

这些为必填项，请通过配置文件或环境变量补齐，见 [完整速查表](#完整速查表)。

## 另见

- [配置文件说明](CONFIG_FILES.md) - `config.toml` 文件本身的使用
- [配置指南](CONFIGURATION.md) - 完整配置项与示例
- [配置模块文档](../server/CONFIG.md) - 配置加载实现与验证规则
- [OIDC 对接部署](OIDC.md) - OIDC 登录接入
- [GeoIP 配置指南](GEOIP.md) - GeoIP 启用步骤
- [Docker 部署](../deployment/DOCKER.md) - 容器化部署
- [Systemd 服务](../deployment/SYSTEMD.md) - systemd 部署
