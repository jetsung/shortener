# Shortener

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-jetsung%2Fshortener-blue)](https://github.com/jetsung/shortener)

使用 Rust 编写的高性能 URL 短链接服务，提供完整的 Web 服务器、前端界面和命令行管理工具。

## ✨ 特性

- 🚀 **高性能**: 基于 Rust 和 async/await，实现最大吞吐量
- 💾 **多数据库**: 支持 SQLite、PostgreSQL 和 MySQL
- ⚡ **缓存加速**: 集成 Redis/Valkey 提升性能
- 🌍 **地理位置**: 使用 ip2region 追踪访客地理信息
- 🔒 **安全认证**: API 密钥和 JWT 令牌双重支持
- 📊 **访问分析**: 全面的访问历史追踪
- 🎨 **现代前端**: 基于 React + Semi Design 的管理界面
- 💻 **CLI 工具**: 易用的命令行管理工具
- 🐳 **容器化**: Docker 和 Docker Compose 支持
- 📦 **跨平台**: 支持 Linux、macOS 和 Windows

## 📦 项目组成

本项目采用 monorepo 结构，包含以下子项目：

| 项目 | 说明 | 技术栈 |
|------|------|--------|
| **shortener-server** | RESTful API 服务器 | Rust + Axum + SeaORM |
| **shortener-frontend** | Web 管理界面 | React + TypeScript + Semi Design |
| **shortener-cli** | 命令行管理工具 | Rust + Clap |
| **shortener-common** | 共享类型和工具 | Rust |

## 🚀 快速开始

### 一键安装（推荐）

```bash
curl -sSL https://raw.githubusercontent.com/jetsung/shortener/main/scripts/install.sh | bash
```

### 使用 Docker

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 启动服务
docker compose -f docker/docker-compose.yml up -d
```

访问 `http://localhost:8080` 开始使用。

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 构建并运行服务器
cargo run --release -p shortener-server

# 在另一个终端，初始化 CLI
cargo run -p shortener-cli -- init

# 创建短链接
cargo run -p shortener-cli -- create https://example.com
```

## 📚 文档

- 📖 **[完整文档](https://jetsung.github.io/shortener)** - 在线文档站点
- 🚀 **[快速开始](general/INSTALLATION.md)** - 安装和配置指南
- 🖥️ **[服务器文档](server/README.md)** - API 服务器使用说明
- 🎨 **[前端文档](frontend/README.md)** - Web 界面使用说明
- 💻 **[CLI 文档](cli/README.md)** - 命令行工具使用说明
- 🐳 **[部署指南](deployment/README.md)** - 生产环境部署
- 🔌 **[API 参考](server/API.md)** - RESTful API 文档

### 本地查看文档

```bash
# 安装 Zensical（推荐 uv，独立 venv；也可用 pip install zensical）
uv tool install zensical

# 启动文档服务器
zensical serve

# 访问 http://127.0.0.1:8000
```

## 🛠️ 开发

### 环境要求

- Rust 1.90+
- Node.js 18+ (前端开发)
- pnpm 8+ (前端开发)

### 开发服务器

```bash
# 后端开发
cargo watch -x 'run -p shortener-server'

# 前端开发
cd shortener-frontend
pnpm dev
```

### 代码质量

```bash
# 格式化
cargo fmt

# 代码检查
cargo clippy -- -D warnings

# 运行测试
cargo test

# 前端测试
cd shortener-frontend
pnpm test
```

## 🤝 贡献

欢迎贡献！请查看 [贡献指南](CONTRIBUTING.md) 了解详情。

## 📄 许可证

本项目采用 Apache-2.0 许可证 - 详见 [LICENSE](../LICENSE) 文件。

## 👤 作者

**Jetsung Chan** <i@jetsung.com>

- GitHub: [@jetsung](https://github.com/jetsung)
- Website: [jetsung.com](https://jetsung.com)

## 🙏 致谢

- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM 框架
- [Semi Design](https://semi.design/) - UI 组件库
- [Clap](https://github.com/clap-rs/clap) - CLI 框架

---

**使用 ❤️ 和 Rust 构建**
