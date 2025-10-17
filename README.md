# Shortener

使用 Rust 编写的高性能 URL 短链接服务，提供 RESTful API 服务器和命令行管理工具。

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-jetsung%2Fshortener-blue)](https://github.com/jetsung/shortener)

## 目录

- [特性](#特性)
- [项目结构](#项目结构)
- [快速开始](#快速开始)
- [安装](#安装)
- [配置](#配置)
- [使用](#使用)
- [API 文档](#api-文档)
- [部署](#部署)
- [开发](#开发)
- [测试](#测试)
- [性能](#性能)

- [贡献](#贡献)
- [许可证](#许可证)

## 特性

- **�� 高性能**: 使用 Rust 和 async/await 构建，实现最大吞吐量
- **💾 多数据库支持**: 支持 SQLite、PostgreSQL 和 MySQL
- **⚡ 缓存**: 集成 Redis/Valkey 提升性能
- **🌍 地理位置**: 使用 ip2region 追踪访客地理信息
- **🔒 安全**: API 密钥认证和 JWT 令牌支持
- **📊 分析**: 全面的访问历史追踪
- **🛠️ RESTful API**: 完整的 URL 管理 API
- **💻 CLI 工具**: 易用的命令行界面
- **🔧 类型安全**: 利用 Rust 的类型系统保证可靠性
- **🐳 Docker 就绪**: 多阶段 Docker 构建和 Docker Compose 支持
- **📦 跨平台**: 支持 Linux、macOS 和 Windows

## 项目结构

本项目使用 Cargo workspace 组织多个相关包：

```
shortener/
├── shortener-server/    # 提供 RESTful API 的 Web 服务器
│   ├── src/
│   │   ├── handlers/    # HTTP 请求处理器
│   │   ├── services/    # 业务逻辑层
│   │   ├── repositories/# 数据访问层
│   │   ├── models/      # 数据模型
│   │   ├── middleware/  # 认证、日志等
│   │   ├── cache/       # 缓存抽象 (Redis/Valkey)
│   │   └── geoip/       # GeoIP 功能
│   └── tests/           # 集成测试
├── shortener-cli/       # 命令行管理工具
│   └── src/
│       ├── commands/    # CLI 命令
│       └── client.rs    # API 客户端
├── shortener-common/    # 共享类型和工具
└── config/              # 配置示例
```

## 快速开始

### 前提条件

- Rust 1.90 或更高版本（从 [rustup.rs](https://rustup.rs/) 安装）
- Cargo（随 Rust 一起安装）

### 构建和运行

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 构建所有包
cargo build --release

# 运行服务器
cargo run --release -p shortener-server

# 在另一个终端中，初始化 CLI
cargo run -p shortener-cli -- init

# 创建短链接
cargo run -p shortener-cli -- create https://example.com
```

## 安装

### 从源码安装

```bash
# 构建发布版本
cargo build --release

# 安装服务器
sudo cp target/release/shortener-server /usr/local/bin/

# 安装 CLI
cargo install --path shortener-cli
```

### 使用 Cargo 安装

```bash
# 安装 CLI 工具
cargo install --path shortener-cli

# 或从 crates.io 安装（发布后）
cargo install shortener-cli
```

### 使用 Docker

```bash
# 拉取镜像（发布后）
docker pull jetsung/shortener-server:latest

# 或本地构建
docker build -f docker/Dockerfile -t shortener-server .

# 使用 Docker Compose 运行
docker compose -f docker/docker-compose.yml up -d
```

### 使用预构建二进制文件

从 [Releases](https://github.com/jetsung/shortener/releases) 页面下载预构建的二进制文件。

```bash
# Linux (x86_64)
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-x86_64-unknown-linux-musl.tar.gz
tar xzf shortener-server-v1.0.0-x86_64-unknown-linux-musl.tar.gz
sudo mv shortener-server /usr/local/bin/

# macOS (Apple Silicon)
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-aarch64-apple-darwin.tar.gz
tar xzf shortener-server-v1.0.0-aarch64-apple-darwin.tar.gz
sudo mv shortener-server /usr/local/bin/
```

### 使用 DEB 包（Debian/Ubuntu）

```bash
# 下载 deb 包
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server_1.0.0_amd64.deb

# 安装
sudo apt install ./shortener-server_1.0.0_amd64.deb

# 或使用 dpkg
sudo dpkg -i shortener-server_1.0.0_amd64.deb
sudo apt-get install -f  # 修复依赖

# 服务会自动启动
sudo systemctl status shortener-server
```

详见 [DEB 打包指南](docs/DEB_PACKAGING.md)。

## 配置

### 服务器配置

在 `config/config.toml` 创建配置文件：

```toml
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "your-secret-api-key"

[shortener]
code_length = 6
code_charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

[admin]
username = "admin"
password = "your-secure-password"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "data/shortener.db"

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

[geoip]
enabled = true
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
```

详见 [配置指南](docs/CONFIGURATION.md)。

### CLI 配置

初始化 CLI 配置：

```bash
shortener-cli init
```

这将创建 `~/.config/shortener/config.toml`：

```toml
[server]
url = "http://localhost:8080"
api_key = "your-api-key"
```

或使用环境变量：

```bash
export SHORTENER_URL="http://localhost:8080"
export SHORTENER_KEY="your-api-key"
```

## 使用

### 服务器

启动服务器：

```bash
# 开发模式
cargo run -p shortener-server

# 生产模式
cargo run --release -p shortener-server

# 使用自定义配置
cargo run -p shortener-server -- --config /path/to/config.toml

# 显示版本
cargo run -p shortener-server -- --version
```

服务器将显示启动信息，包括：

- 服务器版本
- 监听地址
- API 密钥
- 管理员凭据
- 数据库类型
- 缓存状态
- GeoIP 状态

### CLI

```bash
# 显示帮助
shortener-cli --help

# 初始化配置
shortener-cli init

# 显示环境
shortener-cli env

# 创建短链接
shortener-cli create https://example.com

# 使用自定义代码创建
shortener-cli create https://example.com --code mylink --desc "我的链接"

# 获取 URL 详情
shortener-cli get mylink

# 列出所有 URL
shortener-cli list --all

# 分页列出
shortener-cli list --page 1 --psize 20 --sort created_at --order desc

# 更新 URL
shortener-cli update mylink --ourl https://newurl.com --desc "已更新"

# 删除 URL
shortener-cli delete mylink
```

详见 [CLI 文档](shortener-cli/README.md)。

## API 文档

### 认证

所有 API 请求需要使用以下方式之一进行认证：

1. **API 密钥**（Header）：`X-API-KEY: your-api-key`
2. **JWT 令牌**（Header）：`Authorization: Bearer <token>`

### 端点

#### 账户管理

- `POST /api/account/login` - 登录并获取 JWT 令牌
- `POST /api/account/logout` - 登出
- `GET /api/users/current` - 获取当前用户信息

#### 短链接管理

- `POST /api/shortens` - 创建短链接
- `GET /api/shortens` - 列出短链接（分页）
- `GET /api/shortens/{code}` - 获取短链接详情
- `PUT /api/shortens/{code}` - 更新短链接
- `DELETE /api/shortens/{code}` - 删除短链接
- `DELETE /api/shortens?ids=1,2,3` - 批量删除

#### 访问历史

- `GET /api/histories` - 列出访问历史（分页）
- `DELETE /api/histories?ids=1,2,3` - 批量删除历史

### 请求示例

```bash
# 登录
curl -X POST http://localhost:8080/api/account/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'

# 创建短链接
curl -X POST http://localhost:8080/api/shortens \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"original_url":"https://example.com","code":"test"}'

# 获取短链接
curl http://localhost:8080/api/shortens/test \
  -H "X-API-KEY: your-api-key"

# 列出短链接
curl "http://localhost:8080/api/shortens?page=1&page_size=10" \
  -H "X-API-KEY: your-api-key"
```

详见 [API 文档](docs/API.md) 和 [OpenAPI 规范](openapi.yml)。

## 部署

### Docker

详见 [Docker 部署指南](docs/DOCKER.md)。

```bash
# 开发环境
docker compose -f docker/docker-compose.dev.yml up -d

# 生产环境
docker compose -f docker/docker-compose.yml up -d
```

### Systemd

详见 [Systemd 部署指南](deploy/systemd/README.md)。

```bash
# 安装服务
cd deploy/systemd
sudo ./install.sh

# 启动服务
sudo systemctl start shortener-server
sudo systemctl enable shortener-server
```

### 交叉编译

详见 [交叉编译指南](scripts/CROSS_COMPILE.md)。

```bash
# 为多个平台构建
./scripts/build-cross.sh --all
```

## 开发

### 设置开发环境

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装开发工具
cargo install cargo-watch cargo-audit

# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 构建
cargo build
```

### 开发工作流

```bash
# 监视并在更改时重新构建
cargo watch -x 'run -p shortener-server'

# 格式化代码
cargo fmt

# 代码检查
cargo clippy -- -D warnings

# 运行测试
cargo test

# 运行特定测试
cargo test test_name

# 运行测试并输出
cargo test -- --nocapture

# 安全审计
cargo audit
```

### 代码结构

- **Handlers**: HTTP 请求处理（在 `shortener-server/src/handlers/`）
- **Services**: 业务逻辑（在 `shortener-server/src/services/`）
- **Repositories**: 数据访问（在 `shortener-server/src/repositories/`）
- **Models**: 数据结构（在 `shortener-server/src/models/`）
- **Middleware**: 横切关注点（在 `shortener-server/src/middleware/`）

## 测试

### 单元测试

```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p shortener-server
cargo test -p shortener-cli

# 运行测试并生成覆盖率报告（需要 cargo-tarpaulin）
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 集成测试

```bash
# 运行集成测试
cargo test --test integration_test

# 运行 API 集成测试
cargo test --test api_integration_test
```

### 基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench code_generation_bench

# 快速基准测试
cargo bench -- --test
```

详见 [基准测试指南](shortener-server/BENCHMARKS.md)。

## 性能

### 基准测试结果

- **代码生成**: 每个代码约 215 纳秒
- **URL 验证**: 每次验证约 50-100 纳秒
- **数据库操作**: 大多数操作在亚毫秒级
- **缓存操作**: 最小开销

### 优化建议

1. 生产环境启用缓存（Redis/Valkey）
2. 使用 PostgreSQL 或 MySQL 获得更好的并发性能
3. 仅在需要时启用 GeoIP
4. 根据负载调整连接池大小
5. 生产环境使用发布版本（`--release`）

## 贡献

欢迎贡献！请阅读 [贡献指南](CONTRIBUTING.md) 了解详情。

快速开始：

1. Fork 本仓库
2. 创建特性分支（`git checkout -b feature/amazing-feature`）
3. 提交更改（`git commit -m '添加：amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 开启 Pull Request

### 开发指南

- 遵循 Rust 命名约定
- 为新功能编写测试
- 更新文档
- 提交前运行 `cargo fmt` 和 `cargo clippy`
- 确保所有测试通过

详见 [贡献指南](CONTRIBUTING.md)。

## 许可证

本项目采用 Apache-2.0 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- 使用 [Axum](https://github.com/tokio-rs/axum) Web 框架构建
- 数据库 ORM：[SeaORM](https://www.sea-ql.org/SeaORM/)
- CLI 框架：[Clap](https://github.com/clap-rs/clap)

## 文档

本项目使用 MkDocs 构建文档。

### 本地查看文档

```bash
# 安装 MkDocs 和依赖
pip install -r docs/requirements.txt

# 启动本地文档服务器
mkdocs serve

# 在浏览器中访问 http://127.0.0.1:8000
```

### 构建文档

```bash
# 构建静态文档
mkdocs build

# 输出在 site/ 目录
```

## 支持

- 📖 [在线文档](https://jetsung.github.io/shortener)
- 🐛 [问题追踪](https://github.com/jetsung/shortener/issues)

---

**使用 ❤️ 和 Rust 构建**

**作者**: [Jetsung Chan](https://github.com/jetsung) <i@jetsung.com>
