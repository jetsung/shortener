# Docker 高级部署指南

本文档介绍 Shortener 服务的 Docker 高级部署主题，包括多平台镜像构建、性能调优、安全加固与高级排障。

> 基础 Docker 部署（快速开始、Compose 编排、环境变量）请参阅 [Docker 部署指南](DOCKER.md)；
> 端到端完整部署（后端 + 前端 + 数据库 + 反向代理）请参阅 [Docker 完整指南](DOCKER_FULL.md)。

## 目录

- [多平台镜像构建](#多平台镜像构建)
- [Docker Bake 高级用法](#docker-bake-高级用法)
- [镜像体积优化](#镜像体积优化)
- [性能优化](#性能优化)
- [安全加固](#安全加固)
- [多服务编排](#多服务编排)
- [高级排障](#高级排障)

## 多平台镜像构建

### 使用 Cross 交叉编译

项目使用 [Cross](https://github.com/cross-rs/cross) 通过 Docker 交叉编译多平台二进制：

```bash
# 安装 Cross
cargo install cross --git https://github.com/cross-rs/cross

# 为 Linux x86_64（musl）构建服务器
cross build --release --target x86_64-unknown-linux-musl -p shortener-server

# 为 ARM64 构建
cross build --release --target aarch64-unknown-linux-musl -p shortener-server

# 为 Windows 构建
cross build --release --target x86_64-pc-windows-gnu -p shortener-server
```

跨平台目标的完整说明见 [交叉编译指南](CROSS_COMPILE.md)。

### 使用 Docker Buildx

使用 Docker Buildx 构建多平台镜像（在一个命令中产出 linux/amd64 和 linux/arm64）：

```bash
# 创建构建器
docker buildx create --use

# 构建并推送多平台镜像
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t jetsung/shortener-server:latest \
  --push \
  -f docker/Dockerfile.backend .
```

## Docker Bake 高级用法

项目使用 Docker Bake（`docker/docker-bake.hcl`）进行批量构建。

### 本地构建

```bash
# 构建默认镜像
docker buildx bake -f docker/docker-bake.hcl

# 构建开发镜像（amd64 / arm64）
docker buildx bake -f docker/docker-bake.hcl dev-amd64
docker buildx bake -f docker/docker-bake.hcl dev-arm64
docker buildx bake -f docker/docker-bake.hcl dev

# 构建多平台发布镜像
docker buildx bake -f docker/docker-bake.hcl release
```

### 自定义构建

```bash
# 使用自定义标签
docker buildx bake -f docker/docker-bake.hcl --set "*.tags=myregistry/shortener:<version>"

# 推送到仓库
docker buildx bake -f docker/docker-bake.hcl --push release

# 设置平台
docker buildx bake -f docker/docker-bake.hcl --set "*.platform=linux/amd64,linux/arm64"
```

## 镜像体积优化

项目提供基于 Debian 的标准镜像。

### 标准镜像

```bash
docker build -f docker/Dockerfile.backend -t shortener-server:latest .
```

- 基础镜像：`debian:trixie-slim`
- 大小：约 150MB
- 适用于：通用场景，兼容性好

## 性能优化

### 资源限制

为容器设置 CPU / 内存限额，防止资源耗尽：

```yaml
services:
  shortener-server:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
```

### 缓存与压缩

- 后端启用 Redis/Valkey 缓存（`CACHE__ENABLED=true`、`CACHE__URL`）
- 前端启用 Gzip 压缩与静态资源缓存（见 `docker/nginx-frontend.conf`）

### 健康检查

后端服务提供健康检查端点 `/ping`（返回 `{"message":"pong"}`），而非 `/health`。

> 注意：`Dockerfile.backend` 基于 `scratch` 空镜像，不含 curl/wget 等探针工具，因此镜像内未配置 `HEALTHCHECK`。如需容器级健康检查，需自行提供探针（如静态编译的 curl）或在编排层挂载；也可直接使用外部监控探测 `/ping`。

## 安全加固

### 非 root 用户

镜像以非 root 用户运行（`shortener`，UID 1000）：

```dockerfile
USER shortener
```

### 密钥管理

生产环境使用 Docker Secrets 或环境文件注入敏感信息，避免写进镜像或配置文件：

```bash
# 创建 .env 文件
cat > .env << EOF
DATABASE__URL=postgres://shortener:your_secure_password@postgres:5432/shortener
CACHE__URL=redis://:your_redis_password@redis:6379/0
API_KEY=$(openssl rand -base64 32)
OIDC__CLIENT_SECRET=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 48)
EOF

# 使用环境文件
docker compose --env-file .env up -d
```

`JWT_SECRET` 若以文件形式挂载（Docker/K8s Secret），可用 `JWT_SECRET_FILE` 指向挂载路径。

### 网络隔离

仅暴露必要端口：

```yaml
ports:
  - "8080:8080"  # 仅暴露服务器端口
```

生产环境**不应**将数据库和缓存端口暴露到宿主机。

## 多服务编排

结合后端、前端与反向代理的完整编排（如 Caddy / Traefik），详见 [Docker 完整指南](DOCKER_FULL.md) 与 [前端 Docker 部署指南](DOCKER_FRONTEND.md)。

反向代理需正确转发 OIDC 回调路径：

```
https://<域名>/api/oidc/callback
```

## 高级排障

### 查看日志

```bash
# 所有服务
docker compose -f docker/docker-compose.yml logs -f

# 特定服务
docker compose -f docker/docker-compose.yml logs -f shortener-server
```

### 检查容器状态

```bash
docker compose -f docker/docker-compose.yml ps
docker inspect shortener-server | grep -A 10 Health
```

### 在容器中执行命令

```bash
# Shell 访问
docker compose -f docker/docker-compose.yml exec shortener-server sh

# 检查配置
docker compose -f docker/docker-compose.yml exec shortener-server cat /app/config.toml

# 检查数据库连接
docker compose -f docker/docker-compose.yml exec postgres psql -U shortener -d shortener
```

### 重新构建

```bash
# 重新构建并重启
docker compose -f docker/docker-compose.yml up -d --build

# 强制重新创建
docker compose -f docker/docker-compose.yml up -d --force-recreate
```

### 清理

```bash
# 停止并删除容器
docker compose -f docker/docker-compose.yml down

# 同时删除卷
docker compose -f docker/docker-compose.yml down -v

# 删除所有未使用的 Docker 资源
docker system prune -a
```

## 参考

- [Docker 文档](https://docs.docker.com/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [Docker Buildx Bake](https://docs.docker.com/build/bake/)
- [交叉编译指南](CROSS_COMPILE.md)
- [Docker 部署指南](DOCKER.md)