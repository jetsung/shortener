# 配置指南

Shortener 服务器的完整配置参考。

## 目录

- [概述](#概述)
- [配置文件](#配置文件)
- [环境变量](#环境变量)
- [服务器配置](#服务器配置)
- [数据库配置](#数据库配置)
- [缓存配置](#缓存配置)

## 概述

Shortener 服务器使用 TOML 格式进行配置。配置可以从以下位置加载：

1. 配置文件（默认：`config/config.toml`）
2. 环境变量（前缀：`SHORTENER__`）
3. 命令行参数

优先级（从高到低）：命令行 > 环境变量 > 配置文件

## 配置文件

### 位置

默认位置（按优先级顺序）：

1. `--config` 标志指定的路径
2. `./config/config.toml`
3. `/etc/shortener/config.toml`

### 格式

```toml
[server]
# 服务器设置

[shortener]
# 短代码生成设置

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

环境变量使用前缀 `SHORTENER__`，用双下划线分隔嵌套键：

```bash
# 服务器地址
export SHORTENER__SERVER__ADDRESS=":9090"

# 数据库类型
export SHORTENER__DATABASE__TYPE="postgres"

# 启用缓存
export SHORTENER__CACHE__ENABLED="true"

# API 密钥
export SHORTENER__SERVER__API_KEY="your-secret-key"
```

## 服务器配置

```toml
[server]
address = ":8080"                          # 监听地址
site_url = "http://localhost:8080"        # 公共站点 URL
api_key = "your-secret-api-key"           # API 密钥（必需）
```

### 详细说明

- `address`：服务器监听地址，默认 `:8080`
- `site_url`：站点的公共 URL，用于生成短链接
- `api_key`：用于认证的 API 密钥，使用 `openssl rand -base64 32` 生成

## 短链接配置

```toml
[shortener]
code_length = 6                           # 短代码长度（4-16）
code_charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
```

## 管理员配置

```toml
[admin]
username = "admin"                        # 管理员用户名（必需）
password = "your-secure-password"         # 管理员密码（必需）
```

## 数据库配置

### SQLite

```toml
[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "data/shortener.db"
```

### PostgreSQL

```toml
[database]
type = "postgres"
log_level = 1

[database.postgres]
host = "localhost"
port = 5432
user = "postgres"
password = "postgres"
database = "shortener"
sslmode = "disable"
timezone = "Asia/Shanghai"
```

### MySQL

```toml
[database]
type = "mysql"
log_level = 1

[database.mysql]
host = "localhost"
port = 3306
user = "root"
password = "root"
database = "shortener"
charset = "utf8mb4"
```

## 缓存配置

### Redis

```toml
[cache]
enabled = true
type = "redis"
expire = 3600
prefix = "shorten:"

[cache.redis]
host = "localhost"
port = 6379
password = ""
db = 0
```

### Valkey

```toml
[cache]
enabled = true
type = "valkey"
expire = 3600

[cache.valkey]
host = "localhost"
port = 6379
password = ""
db = 0
```

## GeoIP 配置

```toml
[geoip]
enabled = true
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
```

## 配置示例

### 开发环境

```toml
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "dev-api-key"

[admin]
username = "admin"
password = "admin123"

[database]
type = "sqlite"

[database.sqlite]
path = "data/shortener.db"

[cache]
enabled = false

[geoip]
enabled = false
```

### 生产环境

```toml
[server]
address = ":8080"
site_url = "https://short.example.com"
api_key = "${SHORTENER_API_KEY}"

[admin]
username = "${SHORTENER_ADMIN_USER}"
password = "${SHORTENER_ADMIN_PASS}"

[database]
type = "postgres"

[database.postgres]
host = "postgres"
port = 5432
user = "shortener"
password = "${POSTGRES_PASSWORD}"
database = "shortener"
sslmode = "require"

[cache]
enabled = true
type = "redis"

[cache.redis]
host = "redis"
port = 6379
password = "${REDIS_PASSWORD}"

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

- [API 文档](API.md)
- [部署指南](DEPLOYMENT.md)
- [Docker 部署](DOCKER.md)
