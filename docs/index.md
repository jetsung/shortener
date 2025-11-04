# Shortener 文档

欢迎使用 Shortener - 一个使用 Rust 编写的高性能 URL 短链接服务！

## 🎯 项目简介

Shortener 是一个完整的短链接解决方案，包含：

- **服务器端**: 基于 Rust + Axum 的高性能 RESTful API 服务
- **前端界面**: 基于 React + Semi Design 的现代化管理界面
- **CLI 工具**: 功能完整的命令行管理工具
- **多数据库**: 支持 SQLite、PostgreSQL 和 MySQL
- **缓存支持**: 集成 Redis/Valkey 提升性能
- **地理位置**: 使用 ip2region 追踪访客信息

## 🚀 快速开始

### 一键安装

```bash
curl -sSL https://raw.githubusercontent.com/jetsung/shortener/main/scripts/install.sh | bash
```

### Docker 部署

```bash
git clone https://github.com/jetsung/shortener.git
cd shortener
docker compose -f docker/docker-compose.yml up -d
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 构建服务器
cargo build --release -p shortener-server

# 运行服务器
./target/release/shortener-server
```

## 📚 文档导航

### 入门指南

- [安装指南](general/INSTALLATION.md) - 详细的安装说明
- [配置指南](general/CONFIGURATION.md) - 配置选项说明
- [配置文件](general/CONFIG_FILES.md) - 配置文件详解

### 组件文档

- [服务器文档](server/README.md) - API 服务器使用说明
- [前端文档](frontend/README.md) - Web 界面使用说明
- [CLI 文档](cli/README.md) - 命令行工具使用说明

### 部署指南

- [部署概述](deployment/README.md) - 部署方式总览
- [Docker 部署](deployment/DOCKER.md) - 使用 Docker 部署
- [生产部署](deployment/DEPLOYMENT.md) - 生产环境部署
- [DEB 打包](deployment/DEB_PACKAGING_SIMPLIFIED.md) - Debian 包制作

### API 参考

- [API 文档](server/API.md) - RESTful API 完整参考
- [性能基准](server/BENCHMARKS.md) - 性能测试结果

## ✨ 主要特性

### 高性能

- 基于 Rust 和 async/await 构建
- 支持高并发请求处理
- 内置缓存机制
- 优化的数据库查询

### 易于使用

- 简洁的 RESTful API
- 直观的 Web 管理界面
- 功能完整的 CLI 工具
- 详细的文档和示例

### 灵活部署

- 支持多种数据库
- Docker 容器化部署
- 跨平台支持
- 易于扩展

### 安全可靠

- API 密钥认证
- JWT 令牌支持
- 类型安全的代码
- 全面的测试覆盖

## 🛠️ 技术栈

### 后端

- **语言**: Rust 1.90+
- **Web 框架**: Axum
- **ORM**: SeaORM
- **数据库**: SQLite / PostgreSQL / MySQL
- **缓存**: Redis / Valkey
- **认证**: JWT

### 前端

- **框架**: React 19
- **语言**: TypeScript 5
- **UI 库**: Semi Design
- **构建工具**: Vite 7
- **路由**: React Router 7

### CLI

- **语言**: Rust
- **CLI 框架**: Clap
- **HTTP 客户端**: Reqwest

## 📖 使用示例

### 创建短链接

```bash
# 使用 CLI
shortener-cli create https://example.com

# 使用 API
curl -X POST http://localhost:8080/api/shortens \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"original_url":"https://example.com"}'
```

### 获取短链接

```bash
# 使用 CLI
shortener-cli get abc123

# 使用 API
curl http://localhost:8080/api/shortens/abc123 \
  -H "X-API-KEY: your-api-key"
```

### 列出所有短链接

```bash
# 使用 CLI
shortener-cli list --all

# 使用 API
curl "http://localhost:8080/api/shortens?page=1&page_size=10" \
  -H "X-API-KEY: your-api-key"
```

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

- [GitHub 仓库](https://github.com/jetsung/shortener)
- [问题追踪](https://github.com/jetsung/shortener/issues)
- [贡献指南](CONTRIBUTING.md)

## 📄 许可证

本项目采用 Apache-2.0 许可证。详见 [LICENSE](https://github.com/jetsung/shortener/blob/main/LICENSE) 文件。

## 👤 作者

**Jetsung Chan** <i@jetsung.com>

- GitHub: [@jetsung](https://github.com/jetsung)
- Website: [jetsung.com](https://jetsung.com)

---

**使用 ❤️ 和 Rust 构建**
