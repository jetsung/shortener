# Docker 高级部署指南

本文档介绍 Shortener 服务的 Docker 高级部署主题，包括多平台镜像构建、性能调优、安全加固与高级排障。

> 基础 Docker 部署（快速开始、Compose 编排、环境变量）请参阅 [Docker 部署指南](DOCKER.md)；
> 端到端完整部署（统一镜像 + 数据库 + 反向代理）请参阅 [All-In-One Docker 部署](DOCKER_AIO.md)。

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
  -t jetsung/shortener:latest \
  --push \
  -f docker/Dockerfile .
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

项目仅提供统一镜像（`docker/Dockerfile`）。

```bash
docker build -f docker/Dockerfile -t shortener:latest .
```

- 运行时基础镜像：`gcr.io/distroless/static-debian13:nonroot`（单进程，无 shell/包管理器）
- 后端为 musl 静态链接二进制，无语言运行时依赖

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
- 后端已启用静态文件服务与 `Cache-Control` 缓存头；如需更激进的压缩/缓存策略，可在外层网关配置

### 健康检查

后端服务提供健康检查端点 `/go/`（返回 `{"message":"pong"}`），而非 `/health`。

> 注意：镜像为 distroless（无 shell/wget），未内置容器级 `HEALTHCHECK` 探针；健康监控建议使用外部探针请求 `/go/`，或由编排层挂载静态编译的 curl。

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

结合数据库与反向代理的完整编排（如 Caddy / Traefik），详见 [All-In-One Docker 部署指南](DOCKER_AIO.md)。

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