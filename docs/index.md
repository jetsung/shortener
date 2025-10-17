# Shortener 文档

欢迎来到 Shortener 项目文档！Shortener 是一个用 Rust 编写的高性能 URL 短链接服务。

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-jetsung%2Fshortener-blue)](https://github.com/jetsung/shortener)

## 功能特性

- **🚀 高性能**：使用 Rust 和 async/await 构建，实现最大吞吐量
- **💾 多数据库支持**：支持 SQLite、PostgreSQL 和 MySQL
- **⚡ 缓存**：集成 Redis/Valkey 以提升性能
- **🌍 地理位置**：使用 ip2region 跟踪访客地理信息
- **🔒 安全**：API 密钥认证和 JWT 令牌支持
- **📊 分析**：全面的访问历史跟踪
- **🛠️ RESTful API**：完整的 URL 管理 API
- **💻 CLI 工具**：易于使用的命令行界面
- **🔧 类型安全**：利用 Rust 的类型系统确保可靠性
- **🐳 Docker 就绪**：多阶段 Docker 构建和 Docker Compose 支持
- **📦 跨平台**：支持 Linux、macOS 和 Windows

## 文档导航

### 🚀 快速开始

- **[安装指南](INSTALLATION.md)** - 详细的安装说明和多种安装方式
- **[配置指南](CONFIGURATION.md)** - 服务器和 CLI 配置选项

### 📚 使用指南

- **[API 文档](API.md)** - RESTful API 参考和示例
- **[CLI 工具](https://github.com/jetsung/shortener/blob/main/shortener-cli/README.md)** - 命令行工具使用指南

### 🚀 部署

- **[部署指南](DEPLOYMENT.md)** - 生产环境部署最佳实践
- **[Docker 部署](DOCKER.md)** - 使用 Docker 和 Docker Compose
- **[DEB 包安装](DEB_PACKAGING_SIMPLIFIED.md)** - Debian/Ubuntu 系统安装

### 🔧 开发

- **[项目结构](#项目结构)** - 代码组织和架构
- **[开发指南](#开发指南)** - 本地开发环境设置
- **[贡献指南](https://github.com/jetsung/shortener/blob/main/CONTRIBUTING.md)** - 如何参与项目开发

## 项目结构

本项目使用 Cargo workspace 组织多个相关包：

```
shortener/
├── shortener-server/    # 提供 RESTful API 的 Web 服务器
├── shortener-cli/       # 命令行管理工具
├── shortener-common/    # 共享类型和工具
├── config/              # 配置示例
├── docs/                # 项目文档
└── docker/              # Docker 配置文件
```

## 快速开始

1. **安装** - 查看 [安装指南](INSTALLATION.md) 了解多种安装方式
2. **配置** - 参考 [配置指南](CONFIGURATION.md) 设置服务器和 CLI
3. **部署** - 选择适合的 [部署方式](DEPLOYMENT.md)
4. **使用** - 通过 [API](API.md) 或 CLI 管理短链接

## 开发指南

### 设置开发环境

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建项目
cargo build

# 运行测试
cargo test

# 启动开发服务器
cargo run -p shortener-server
```

### 代码结构

- **shortener-server/src/handlers/** - HTTP 请求处理器
- **shortener-server/src/services/** - 业务逻辑层
- **shortener-server/src/repositories/** - 数据访问层
- **shortener-server/src/models/** - 数据模型
- **shortener-cli/src/** - CLI 工具实现

## 许可证

本项目采用 Apache-2.0 许可证 - 详见 [LICENSE](https://github.com/jetsung/shortener/blob/main/LICENSE) 文件。

## 支持

- 📖 [在线文档](https://jetsung.github.io/shortener)
- 🐛 [问题追踪](https://github.com/jetsung/shortener/issues)
- 💬 [讨论区](https://github.com/jetsung/shortener/discussions)

---

**用 ❤️ 和 Rust 制作**

**作者**：[Jetsung Chan](https://github.com/jetsung) <i@jetsung.com>
