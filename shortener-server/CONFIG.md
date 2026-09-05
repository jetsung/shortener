# 配置模块文档

## 概述

配置模块为 Shortener 服务器提供了一种健壮且类型安全的方式来管理应用程序设置。它支持从 TOML 文件和环境变量加载配置，具有自动验证和默认值处理功能。

## 特性

- **类型安全配置**：所有配置值都是强类型的
- **多数据源**：从 TOML 文件和环境变量加载
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

```toml
[admin]
username = "admin"                        # 管理员用户名（必需）
password = "secure-password"              # 管理员密码（必需）
```

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

可以使用环境变量覆盖配置（双下划线 `__` 分隔嵌套键，无前缀）：

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
   - `admin.password` 不能为空

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
- `cache.expire`: `3600`
- `cache.prefix`: `shorten:`
- `database.log_level`: `1`

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

## 迁移指南（结构化配置 → URL）

旧版本使用 `type` + 子段（如 `[database.sqlite]`、`[cache.redis]`）描述连接。现已统一为单个 URL，引擎由 scheme 推断：

**数据库**

```toml
# 旧写法
[database]
type = "postgres"
[database.postgres]
host = "localhost"
port = 5432
user = "shortener"
password = "secret"
database = "shortener"
sslmode = "require"

# 新写法（等价）
[database]
url = "postgres://shortener:secret@localhost:5432/shortener?sslmode=require"
```

```toml
# 旧写法
[database]
type = "sqlite"
[database.sqlite]
path = "data/shortener.db"

# 新写法（等价）
[database]
url = "sqlite://data/shortener.db?mode=rwc"
```

**缓存**

```toml
# 旧写法
[cache]
enabled = true
type = "redis"
[cache.redis]
host = "localhost"
port = 6379
password = "secret"
db = 0

# 新写法（等价）
[cache]
enabled = true
url = "redis://:secret@localhost:6379/0"
```

环境变量同样适用：`DATABASE__URL` / `CACHE__URL` 会覆盖配置文件中的 `url` 字段。

## 示例配置文件

请参阅项目根目录中的 `config.toml` 以获取完整的示例配置文件。
