# 配置文件说明

## 配置文件列表

- `config.example.toml` - 完整的配置示例和文档（推荐阅读）
- `config.local.toml` - 本地开发配置（git 已忽略）
- `config.development.toml` - 开发环境预设
- `config.production.toml` - 生产环境配置模板
- `config.docker.toml` - Docker 部署配置
- `test-config.toml` - 测试专用配置

## 快速开始

### 本地开发

1. 复制并编辑本地配置：
   ```bash
   cp config/config.example.toml config/config.local.toml
   # 编辑 config.local.toml，修改 api_key 等敏感信息
   ```

2. 运行服务：
   ```bash
   cargo run -p shortener-server -- --config config/config.local.toml
   ```

### 使用预设配置

直接使用开发环境预设：
```bash
cargo run -p shortener-server -- --config config/config.development.toml
```

## 配置优化

### 可选字段

当某些功能被禁用时，相关的配置字段可以省略：

```toml
# 禁用缓存时，可以省略 cache.type 和具体配置
[cache]
enabled = false

# 禁用 GeoIP 时，可以省略 geoip.type 和具体配置
[geoip]
enabled = false
```

### 环境变量

支持通过环境变量覆盖配置（前缀 `SHORTENER__`）：

```bash
export SHORTENER__SERVER__API_KEY="your-secret-key"
export SHORTENER__ADMIN__PASSWORD="secure-password"
cargo run -p shortener-server
```

## Git 忽略规则

以下文件会被 git 忽略，可以安全地存储敏感信息：
- `*.local.toml`
- `config/config.toml`
