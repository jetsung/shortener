# Shortener Server 文档

本目录包含 shortener-server 的相关文档。

## 文档列表

- [API 文档](API.md) - RESTful API 参考和示例
- [重构说明](REFACTORING.md) - 按照 OpenAPI 规范进行的重构详情

## 快速链接

### 服务器相关
- [配置说明](../general/CONFIGURATION.md)
- [部署指南](../deployment/DEPLOYMENT.md)
- [Docker 部署](../deployment/DOCKER.md)

### 开发相关
- [安装指南](../general/INSTALLATION.md)
- [贡献指南](../../CONTRIBUTING.md)

### 迁移相关
- [API 变更说明](../migration/API_CHANGES.md)
- [数据库迁移](../migration/DATABASE_MIGRATION.md)

## 项目结构

```
shortener-server/
├── src/
│   ├── cache/          # 缓存实现
│   ├── config/         # 配置管理
│   ├── db/             # 数据库连接
│   ├── errors/         # 错误处理
│   ├── geoip/          # GeoIP 功能
│   ├── handlers/       # HTTP 处理器
│   ├── logging/        # 日志配置
│   ├── middleware/     # 中间件
│   ├── migration/      # 数据库迁移
│   ├── models/         # 数据模型
│   ├── repositories/   # 数据访问层
│   ├── services/       # 业务逻辑层
│   ├── router.rs       # 路由配置
│   ├── lib.rs          # 库入口
│   └── main.rs         # 程序入口
├── tests/              # 测试文件
├── benches/            # 性能测试
├── docs/               # 项目文档
└── Cargo.toml          # 项目配置
```

## 技术栈

- **Web 框架**: Axum
- **数据库 ORM**: SeaORM
- **异步运行时**: Tokio
- **序列化**: Serde
- **日志**: Tracing

## 支持的数据库

- SQLite
- PostgreSQL
- MySQL

## 支持的缓存

- Redis
- Valkey
- 内存缓存（NullCache）

## 特性

- ✅ RESTful API
- ✅ 自定义短码
- ✅ 访问统计
- ✅ GeoIP 定位
- ✅ 用户代理解析
- ✅ API 密钥认证
- ✅ JWT 令牌认证
- ✅ 健康检查
- ✅ 缓存支持
- ✅ 多数据库支持

## 开发

### 运行测试

```bash
cargo test
```

### 运行服务器

```bash
cargo run --release
```

### 构建

```bash
cargo build --release
```

## 相关项目

- [shortener-cli](../cli/USAGE.md) - 命令行工具
