# 配置文件说明

Shortener 使用单一配置文件 `config.toml`，位于项目根目录（默认路径），也支持通过 `--config` 参数指定其他路径。配置文件是**可选的**：不提供文件时服务直接以环境变量（加内置缺省值）启动，详见 [环境变量参考](ENVIRONMENT_VARIABLES.md)。

环境变量与文件同时使用时，**环境变量优先**（如 `SERVER__API_KEY` 覆盖文件中的 `api_key`）。

## 快速开始

直接编辑项目根目录的 `config.toml`，修改 `api_key` 等敏感信息后启动服务：

```bash
cargo run -p shortener-server
```

或显式指定配置文件：

```bash
cargo run -p shortener-server -- --config config.toml
```

或完全不使用配置文件（必填项用环境变量提供）：

```bash
export SERVER__API_KEY="your-secret-key"
export ADMIN__PASSWORD_HASH='$argon2id$...'
export DATABASE__URL="sqlite://data/shortener.db?mode=rwc"
export JWT_SECRET="$(openssl rand -base64 48)"
cargo run -p shortener-server
```

## 初始化配置

运行以下命令生成一份新的默认配置：

```bash
cargo run -p shortener-server -- init
```

## 配置优化

### 可选字段

当某些功能被禁用时，相关的配置字段可以省略：

```toml
# 禁用缓存时，可以省略 cache.url
[cache]
enabled = false

# 禁用 GeoIP 时，可以省略 geoip.type 和具体配置
[geoip]
enabled = false
```

### 环境变量

支持通过环境变量覆盖配置（双下划线 `__` 分隔嵌套键，无前缀）：

```bash
export SERVER__API_KEY="your-secret-key"
export ADMIN__PASSWORD_HASH="$argon2id$..."
cargo run -p shortener-server
```

> 完整的环境变量清单与用法（含 JWT_SECRET、各部署方式示例），请参阅 [环境变量参考](ENVIRONMENT_VARIABLES.md)。

## Git 忽略规则

以下文件会被 git 忽略，可以安全地存储敏感信息：
- `*.local.toml`
- `.env`
