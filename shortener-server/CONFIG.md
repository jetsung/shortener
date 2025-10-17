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
site_url = "http://localhost:8080"        # 公共站点 URL
api_key = "your-secret-api-key"           # 用于认证的 API 密钥（必需）
```

### 短链接配置

```toml
[shortener]
code_length = 6                           # 生成的短代码长度（4-16）
code_charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
```

### 管理员配置

```toml
[admin]
username = "admin"                        # 管理员用户名（必需）
password = "secure-password"              # 管理员密码（必需）
```

### 数据库配置

#### SQLite

```toml
[database]
type = "sqlite"
log_level = 1                             # 1=静默, 2=错误, 3=警告, 4=信息

[database.sqlite]
path = "data/shortener.db"
```

#### PostgreSQL

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

#### MySQL

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
parse_time = true
loc = "Local"
```

### 缓存配置

#### Redis

```toml
[cache]
enabled = true
type = "redis"
expire = 3600                             # 缓存过期时间（秒）
prefix = "shorten:"                       # 缓存键前缀

[cache.redis]
host = "localhost"
port = 6379
password = ""
db = 0
```

#### Valkey

```toml
[cache]
enabled = true
type = "valkey"
expire = 3600
prefix = "shorten:"

[cache.valkey]
host = "localhost"
port = 6379
username = ""
password = ""
db = 0
```

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

// 从默认位置加载（config/config.toml）
let config = Config::load()?;

// 从指定文件加载
let config = Config::from_file("path/to/config.toml")?;
```

### 环境变量

可以使用前缀为 `SHORTENER__` 的环境变量覆盖配置：

```bash
# 覆盖服务器地址
export SHORTENER__SERVER__ADDRESS=":9090"

# 覆盖数据库类型
export SHORTENER__DATABASE__TYPE="postgres"

# 覆盖缓存启用
export SHORTENER__CACHE__ENABLED="true"
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
   - `shortener.code_length` 必须在 4 到 16 之间
   - `shortener.code_charset` 不能为空

3. **条件要求**：
   - 当 `database.type = "sqlite"` 时，需要 `database.sqlite` 部分
   - 当 `database.type = "postgres"` 时，需要 `database.postgres` 部分
   - 当 `database.type = "mysql"` 时，需要 `database.mysql` 部分
   - 当 `cache.enabled = true` 且 `cache.type = "redis"` 时，需要 `cache.redis` 部分
   - 当 `cache.enabled = true` 且 `cache.type = "valkey"` 时，需要 `cache.valkey` 部分
   - 当 `geoip.enabled = true` 时，需要 `geoip.ip2region` 部分

## 默认值

如果未指定，将应用以下默认值：

- `server.address`: `:8080`
- `server.site_url`: `http://localhost:8080`
- `shortener.code_length`: `6`
- `shortener.code_charset`: `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ`
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

## 示例配置文件

请参阅项目根目录中的 `config/config.toml` 以获取完整的示例配置文件。
