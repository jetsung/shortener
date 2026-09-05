# 配置模块文档

## 概述

配置模块为 Shortener 服务器提供了一种健壮且类型安全的方式来管理应用程序设置。它支持从 TOML 文件和环境变量加载配置，具有自动验证和默认值处理功能。

## 特性

- **类型安全配置**：所有配置值都是强类型的
- **多数据源**：从 TOML 文件和环境变量加载（**文件可选**：文件不存在时以纯环境变量启动）
- **加载优先级**：环境变量（`__` 形式）> 平铺别名（`DATABASE_URL` 等）> 配置文件 > 缺省值
- **验证**：自动验证必需字段和值范围
- **默认值**：可选配置的合理默认值
- **多数据库支持**：SQLite、PostgreSQL 和 MySQL
- **多缓存后端**：Redis 和 Valkey
- **GeoIP 支持**：ip2region 集成

## 配置结构

### 服务器配置

```toml
[server]
address = ":8080"                          # 服务器监听地址
trusted-platform = ""                      # 可信平台头（可选）
short_url = "https://s.example.com"      # 短址专用域名（可选，未设置时从监听地址推断，通配地址回退 localhost）
api_key = "your-secret-api-key"           # 用于认证的 API 密钥（必需）
```

### 短链接配置

```toml
[slug]
length = 6                           # 生成的短代码长度（4-16）
alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
```

### 管理员配置

管理员账号用于密码登录通道。口令以 **Argon2id 哈希**（PHC 字符串格式）存储，不以明文保存。

```toml
[admin]
username = "admin"                        # 管理员用户名（可选，默认 "admin"）
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$..."  # 口令 Argon2id 哈希（必需）
```

生成哈希（任选其一，交互式输入可在 shell 历史中不留痕）：

```bash
# 服务端自带的子命令（推荐，无需额外安装 CLI）
shortener-server hash-password --password "your-secure-password"
# 或使用命令行工具
shortener-cli hash-password --password "your-secure-password"
```

将输出的整行（`$argon2id$...`）粘贴为 `password_hash` 的值。

### OIDC 配置

对接任意标准 OIDC / OAuth2.0 身份提供方（IdP），仅白名单内的 IdP 用户可登录。`enabled` 为 OIDC 登录的总开关，`issuer` 为空时视为未配置。

```toml
[oidc]
enabled = false                                          # OIDC 总开关（false 时登录不可用）
issuer = "https://keycloak.example.com/realms/shortener"  # IdP 的 issuer 地址（enabled 时必填）
client_id = "shortener-app"                                 # 在 IdP 注册的客户端 ID
client_secret = ""                                          # 客户端密钥（可留空或用 OIDC__CLIENT_SECRET 注入）
# 回调地址由请求 Host 自动推导为 https://<域名>/api/oidc/callback，无需配置
allow_emails = ["admin@example.com"]                        # 允许的邮箱白名单（至少配置一项）
allow_subjects = []                                          # 允许的 sub 白名单
```

敏感配置优先使用环境变量注入（双下划线 `__` 分隔嵌套键）：

```bash
export OIDC__CLIENT_SECRET="your-client-secret"
export JWT_SECRET="$(openssl rand -base64 48)"
# 或以文件形式挂载时：
export JWT_SECRET_FILE=/run/secrets/jwt_secret
```

详细说明请参阅 [OIDC 对接部署](../general/OIDC.md)。

### 数据库配置

数据库连接通过单个 URL 配置，引擎类型（sqlite / postgres / postgresql / mysql）由 URL 的 scheme 自动推断。

```toml
[database]
url = "sqlite://data/shortener.db?mode=rwc"  # 连接 URL（必需）
log_level = 1                                # 1=静默, 2=错误, 3=警告, 4=信息
```

不同引擎的 `url` 示例：

| 引擎 | URL 示例 |
| --- | --- |
| SQLite（文件） | `sqlite://data/shortener.db?mode=rwc` |
| SQLite（内存，仅测试） | `sqlite::memory:` |
| PostgreSQL | `postgres://user:pass@localhost:5432/shortener?sslmode=disable` |
| MySQL | `mysql://user:pass@localhost:3306/shortener?charset=utf8mb4` |

### 缓存配置

缓存连接同样通过 URL 配置，引擎（redis / valkey）由 scheme 推断。

```toml
[cache]
enabled = true
url = "redis://:password@localhost:6379/0"   # 连接 URL
expire = 3600                                # 缓存过期时间（秒）
prefix = "shorten:"                          # 缓存键前缀
```

缓存 URL 示例：

| 引擎 | URL 示例 |
| --- | --- |
| Redis | `redis://:password@localhost:6379/0` |
| Valkey | `valkey://:password@localhost:6379/0` |

#### 缓存行为

- **键结构**：`{prefix}url:{short_code}`（前缀默认 `shorten:`），值为短链 JSON，写入即带 TTL
- **写入时机**：创建、更新短链时写入；访问短链时未命中则查库并回填（惰性回填）
- **删除同步**：删除单条或批量删除短链时，同步删除对应缓存键
- **启动重建**：服务启动时先清空前缀下的所有旧键，再从数据库预热全量短链，保证缓存与数据库一致；该过程完成前服务不对外监听
- **手动刷新**：调用 `POST /api/cache/refresh` 可随时清空并重建缓存（管理界面「短址列表」页也有「刷新缓存」按钮）
- **隔离性**：清空操作仅影响本服务前缀开头的键，同一 Redis/Valkey 实例中其他应用的键不受影响

### GeoIP 配置

```toml
[geoip]
enabled = true
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"                             # "4" 表示 IPv4，"6" 表示 IPv6
```

## 使用

### 加载配置

```rust
use config::Config;

// 从默认位置加载（config.toml）
let config = Config::load()?;

// 从指定文件加载
let config = Config::from_file("path/to/config.toml")?;
```

### 环境变量

可以使用环境变量覆盖配置（双下划线 `__` 分隔嵌套键，无前缀），**环境变量优先于配置文件**；配置文件不存在时以纯环境变量启动：

```bash
# 覆盖服务器地址
export SERVER__ADDRESS=":9090"

# 通过环境变量设置数据库连接 URL
export DATABASE__URL="postgres://user:pass@localhost:5432/shortener?sslmode=require"

# 通过环境变量设置缓存连接 URL
export CACHE__URL="redis://:password@localhost:6379/0"

# 覆盖缓存启用
export CACHE__ENABLED="true"
```

注意：使用双下划线（`__`）分隔嵌套的配置键。

### 获取连接字符串

```rust
// 获取数据库连接 URL
let db_url = config.get_database_url();
// 返回: "sqlite://data/shortener.db?mode=rwc"
//   或: "postgres://user:pass@host:port/db?sslmode=disable"
//   或: "mysql://user:pass@host:port/db?charset=utf8mb4"

// 获取缓存连接 URL（如果启用）
if let Some(cache_url) = config.get_cache_url() {
    // 返回: "redis://:password@host:port/db"
    //   或: "redis://username:password@host:port/db"
}
```

## 验证规则

配置模块自动验证：

1. **必需字段**：
   - `server.api_key` 不能为空
   - `admin.username` 不能为空
   - `admin.password_hash` 不能为空

2. **值范围**：
   - `slug.length` 必须在 4 到 16 之间
   - `slug.alphabet` 不能为空

3. **条件要求**：
   - `database.url` 不能为空（通过 `[database] url` 或 `DATABASE__URL` 设置）
   - 当 `cache.enabled = true` 时，`cache.url` 不能为空（通过 `[cache] url` 或 `CACHE__URL` 设置）
   - 当 `geoip.enabled = true` 时，需要 `geoip.ip2region` 部分

## 默认值

如果未指定，将应用以下默认值：

- `server.address`: `:8080`
- `server.short_url`: 空（从监听地址推断，通配地址回退 localhost）
- `slug.length`: `6`
- `slug.alphabet`: `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`
- `cache.enabled`: `false`
- `cache.expire`: `3600`
- `cache.prefix`: `shorten:`
- `database.log_level`: `1`
- `geoip.enabled`: `false`
- `logging.level`: `info`
- `logging.format`: `json`

## 日志配置

```toml
[logging]
level = "info"              # error / warn / info / debug / trace
format = "json"             # json / pretty / compact
with_timestamp = true
with_target = true
with_thread_ids = false
with_thread_names = false
with_file = false
with_line_number = false
with_ansi = true
```

## 错误处理

配置加载可能因多种原因失败：

```rust
match Config::load() {
    Ok(config) => {
        // 配置加载成功
    }
    Err(e) => {
        // 处理错误
        eprintln!("加载配置失败: {}", e);

        // 常见错误：
        // - 文件未找到
        // - 无效的 TOML 语法
        // - 缺少必需字段
        // - 无效的值范围
        // - 类型不匹配
    }
}
```

## 测试

配置模块包含全面的单元测试，涵盖：

- 有效配置加载
- 默认值应用
- 必需字段验证
- 值范围验证
- 数据库类型配置
- 缓存配置
- GeoIP 配置
- 连接字符串生成
- 错误情况

运行测试：

```bash
cargo test -p shortener-server
```

## 示例配置文件

请参阅项目根目录中的 `config.toml` 以获取完整的示例配置文件。
