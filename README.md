# Shortener

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-jetsung%2Fshortener-blue)](https://github.com/jetsung/shortener)

高性能 URL 短链接服务，Rust 后端 + React 前端。

## ✨ 特性

- 🚀 高性能 Rust 实现
- 💾 支持 SQLite、PostgreSQL、MySQL
- ⚡ Redis/Valkey 缓存加速（启动预热、增删同步、一键刷新）
- 🌍 地理位置追踪
- 🔒 API 密钥和 JWT 认证
- 🎨 React 管理界面
- 💻 命令行工具
- 🐳 Docker 支持

## 📦 项目结构

```
shortener/
├── shortener-server/      # RESTful API 服务器
├── shortener-frontend/    # React 管理界面
├── shortener-cli/         # 命令行工具
├── shortener-common/      # 共享库
├── config.toml            # 服务器配置文件（默认）
├── docker/                # Docker 配置
└── docs/                  # 完整文档
```

## 🚀 快速开始

### Docker 部署（推荐）

```bash
git clone https://github.com/jetsung/shortener.git
cd shortener
docker compose -f docker/docker-compose.yml up -d
```

访问 http://localhost:8080

### 从源码构建

```bash
# 构建服务器
cargo build --release -p shortener-server

# 运行服务器
cargo run --release -p shortener-server

# 使用 CLI
cargo run -p shortener-cli -- create https://example.com
```

## 📚 文档

- 📖 [在线文档](https://jetsung.github.io/shortener)
- 🚀 [安装指南](docs/general/INSTALLATION.md)
- 🔧 [配置指南](docs/general/CONFIGURATION.md)
- 🖥️ [服务器文档](docs/server/README.md)
- 🎨 [前端文档](docs/frontend/README.md)
- 💻 [CLI 文档](docs/cli/README.md)
- 🐳 [部署指南](docs/deployment/README.md)
- 🔌 [API 文档](docs/server/API.md)

### 本地查看文档

```bash
# 使用 uv 安装（推荐，独立 venv，不污染全局环境）
uv tool install zensical

# 或使用 pip 安装
pip install zensical

zensical serve
# 访问 http://127.0.0.1:8000
```

## 🛠️ 开发

```bash
# 后端开发
cargo watch -x 'run -p shortener-server'

# 前端开发
cd shortener-frontend && pnpm dev

# 代码检查
cargo fmt && cargo clippy

# 运行测试
cargo test
```

## 📄 许可证

Apache-2.0 License - 详见 [LICENSE](LICENSE)

## 仓库镜像

[MyCode](https://git.jetsung.com/jetsung/shortener) ● [AtomGit](https://atomgit.com/jetsung/shortener) ● [GitHub](https://github.com/jetsung/shortener)
