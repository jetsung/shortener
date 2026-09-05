# 快速参考

Shortener 项目的常用命令快速参考。

## 📦 安装

```bash
# 从源码构建
cargo build --release

# 安装 CLI
cargo install --path shortener-cli

# Docker
docker compose up -d
```

## 🚀 运行

```bash
# 服务器
cargo run -p shortener-server

# CLI
shortener-cli --help
```

## 📚 文档

```bash
# 查看文档
make docs              # 或 just docs

# 构建文档
make docs-build        # 或 just docs-build

# 部署文档
just docs-deploy
```

## 🔨 开发

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test

# 监视更改
cargo watch -x 'run -p shortener-server'
```

## 🐳 Docker

```bash
# 构建镜像
make build             # 或 just docker-build

# 运行（生产）
make run               # 或 just docker-run

# 运行（开发）
make run-dev           # 或 just docker-run-dev

# 查看日志
make logs              # 或 just docker-logs

# 停止
make stop              # 或 just docker-stop
```

## 🧪 测试

```bash
# 所有测试
cargo test

# 特定包
cargo test -p shortener-server

# 基准测试
cargo bench
```

## 📖 CLI 命令

```bash
# 初始化
shortener-cli init

# 创建短链接
shortener-cli create https://example.com

# 获取详情
shortener-cli get <code>

# 列出所有
shortener-cli list --all

# 更新
shortener-cli update <code> --ourl <new-url>

# 删除
shortener-cli delete <code>
```

## 🔧 配置

```bash
# 服务器配置（config.toml 可选；不提供时纯环境变量启动，env 优先于文件）
vim config.toml

# 纯环境变量最小配置
export SERVER__API_KEY="$(openssl rand -base64 32)"
export ADMIN__PASSWORD_HASH='$argon2id$...'   # shortener-server hash-password --password "..."
export DATABASE__URL="sqlite://data/shortener.db?mode=rwc"
export JWT_SECRET="$(openssl rand -base64 48)"

# CLI 配置
shortener-cli init
vim ~/.config/shortener/config.toml
```

## 🌐 API

```bash
# 登录
curl -X POST http://localhost:8080/api/account/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password"}'

# 创建短链接
curl -X POST http://localhost:8080/api/shortens \
  -H "X-API-KEY: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"original_url":"https://example.com"}'

# 获取短链接
curl http://localhost:8080/api/shortens/<code> \
  -H "X-API-KEY: your-api-key"

# 刷新缓存（清空前缀旧键并从数据库重载）
curl -X POST http://localhost:8080/api/cache/refresh \
  -H "X-API-KEY: your-api-key"
```

## 🔐 安全

```bash
# 生成 API 密钥
openssl rand -base64 32

# 安全审计
cargo audit
```

## 📊 性能

```bash
# 基准测试
cargo bench

# 性能分析
cargo build --release
time ./target/release/shortener-server
```

## 🛠️ 部署

```bash
# Systemd
cd deploy/systemd
sudo ./install.sh
sudo systemctl start shortener-server

# Docker
docker compose -f docker/docker-compose.yml up -d

# 交叉编译
./scripts/build-cross.sh --all
```

## 📝 贡献

```bash
# Fork 并克隆
git clone https://github.com/你的用户名/shortener.git

# 创建分支
git checkout -b feature/your-feature

# 提交更改
git commit -m "添加：your feature"

# 推送
git push origin feature/your-feature
```

## 🔗 链接

- 📖 [完整文档](https://jetsung.github.io/shortener)
- 🐛 [问题追踪](https://github.com/jetsung/shortener/issues)
- 💬 [讨论区](https://github.com/jetsung/shortener/discussions)
- 📧 [邮箱](mailto:i@jetsung.com)

## 📋 检查清单

### 开发前

- [ ] 安装 Rust 1.90+
- [ ] 克隆仓库
- [ ] 安装开发工具
- [ ] 阅读贡献指南

### 提交前

- [ ] 运行 `cargo fmt`
- [ ] 运行 `cargo clippy`
- [ ] 运行 `cargo test`
- [ ] 更新文档
- [ ] 编写提交信息

### 部署前

- [ ] 运行所有测试
- [ ] 更新版本号
- [ ] 构建发布版本
- [ ] 测试部署

---

**提示**: 使用 `just --list` 或 `make help` 查看所有可用命令。
