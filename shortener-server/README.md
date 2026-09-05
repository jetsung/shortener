# Shortener Server

基于 Rust 和 Axum 的高性能 RESTful API 服务器。

## 📚 完整文档

请查看 [服务器文档](../docs/server/README.md) 获取完整的使用说明。

## 🚀 快速开始

```bash
# 构建
cargo build --release -p shortener-server

# 运行
cargo run --release -p shortener-server

# 使用配置文件
cargo run --release -p shortener-server -- --config config.toml
```

## 🛠️ 技术栈

- Rust 1.90+
- Axum (Web 框架)
- SeaORM (ORM)
- SQLite / PostgreSQL / MySQL
- Redis / Valkey (缓存)

## 📖 更多信息

- [完整文档](../docs/server/README.md)
- [API 文档](../docs/server/API.md)
- [配置说明](../docs/server/CONFIG.md)
- [性能基准](../docs/server/BENCHMARKS.md)
- [在线文档](https://jetsung.github.io/shortener/server/)
