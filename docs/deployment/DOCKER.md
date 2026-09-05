# Docker 部署指南

本指南说明如何使用 Docker 和 Docker Compose 部署 Shortener 服务器。

## 快速开始

### 开发环境（SQLite + Redis）

```bash
# 构建并运行
docker compose -f docker/docker-compose.dev.yml up -d

# 查看日志
docker compose -f docker/docker-compose.dev.yml logs -f

# 停止
docker compose -f docker/docker-compose.dev.yml down
```

### 生产环境（PostgreSQL + Redis）

```bash
# 构建并运行
docker compose -f docker/docker-compose.yml up -d

# 查看日志
docker compose -f docker/docker-compose.yml logs -f

# 停止
docker compose -f docker/docker-compose.yml down
```

## Docker 镜像

### 标准镜像（基于 Debian）

```bash
docker build -f docker/Dockerfile.backend -t shortener-server:latest .
```

- 基础镜像：`debian:trixie-slim`
- 大小：约 150MB
- 适用于：通用场景，兼容性好

### All-In-One 镜像（前端 + 后端，单镜像）

```bash
# 构建 All-In-One 镜像
docker build -f docker/Dockerfile -t shortener:latest .

# 或使用 Makefile / Just
make build-aio
just docker-build-aio
```

AIO 镜像同时包含前端与后端，对外仅暴露 **80 端口**（nginx），内部架构：

- **nginx**（监听 80）：托管前端静态资源（前端为 hash 路由 `/#/...`），并反向代理 `/api/*` 与短码重定向到容器内后端
- **shortener-server**（监听 `127.0.0.1:8080`）：API 服务，仅容器内网可访问

两个进程由 `docker/entrypoint-aio.sh` 同时拉起（后端以 `setsid` 独立会话运行，nginx 前台运行）。

快速运行（含 Redis）：

```bash
docker compose -f docker/docker-compose.aio.yml up -d
```

> 注意：`/{short_code}` 短码重定向由 nginx 反代到后端。前端采用 hash 路由（`/#/...`），SPA 路由不占用服务端路径，与短码路径无冲突。

## 使用 Docker Bake

Docker Bake 提供了更强大的构建配置。

### 本地构建

```bash
# 构建默认镜像
docker buildx bake -f docker/docker-bake.hcl

# 构建开发镜像（amd64）
docker buildx bake -f docker/docker-bake.hcl dev-amd64

# 构建开发镜像（arm64）
docker buildx bake -f docker/docker-bake.hcl dev-arm64

# 构建所有开发镜像
docker buildx bake -f docker/docker-bake.hcl dev
```

### 发布构建

```bash
# 构建多平台发布镜像
docker buildx bake -f docker/docker-bake.hcl release

# 这将构建 linux/amd64 和 linux/arm64 平台的镜像
```

### 自定义构建

```bash
# 使用自定义标签
# 自定义标签
docker buildx bake -f docker/docker-bake.hcl --set "*.tags=myregistry/shortener:<version>"

# 推送到仓库
docker buildx bake -f docker/docker-bake.hcl --push release

# 设置平台
docker buildx bake -f docker/docker-bake.hcl --set "*.platform=linux/amd64,linux/arm64"
```

### All-In-One 镜像构建

AIO 镜像同时包含前端与后端，使用 `aio-*` targets：

```bash
# 构建本地开发镜像
docker buildx bake -f docker/docker-bake.hcl aio-default

# 构建多平台开发镜像
docker buildx bake -f docker/docker-bake.hcl aio-dev

# 构建并推送多平台发布镜像
docker buildx bake -f docker/docker-bake.hcl --push aio-release
```

## 配置

### 环境变量

可以在 `docker-compose.yml` 中设置以下环境变量。

#### 服务器配置

- `RUST_LOG`：日志级别（debug、info、warn、error）

其余服务器配置项使用双下划线 `__` 分隔嵌套键（由 `config` crate 自动映射）：

- `SERVER__ADDRESS`：监听地址（如 `:8080`）
- `SERVER__SHORT_URL`：短址专用域名（可选，未设置时从监听地址推断，通配地址回退 localhost）
- `SERVER__API_KEY`：API 密钥

#### 数据库配置

- `DATABASE__URL`：数据库连接字符串（`sqlite://...`、`postgres://...`、`mysql://...`），引擎类型由 URL scheme 推断

#### 缓存配置

- `CACHE__ENABLED`：启用缓存（true/false）
- `CACHE__URL`：缓存连接字符串（`redis://...`、`valkey://...`），引擎类型由 URL scheme 推断
- `CACHE__EXPIRE`：缓存过期时间（秒）

#### GeoIP 配置

- `GEOIP__ENABLED`：启用 GeoIP（true/false）
- `GEOIP__IP2REGION__PATH`：ip2region 数据库路径

#### 认证相关

- `OIDC__ENABLED`：OIDC 登录总开关（true/false）
- `OIDC__CLIENT_SECRET`：OIDC 客户端密钥（覆盖 `[oidc] client_secret`）
- `JWT_SECRET`：JWT 签名密钥（登录签发令牌必需）
- `JWT_SECRET_FILE`：以文件形式挂载的 JWT 密钥路径

### 卷挂载

```yaml
volumes:
  - ../config.toml:/app/config.toml:ro # 配置文件
  - ../data:/app/data # 数据文件（数据库、GeoIP等）
  - app-logs:/app/logs # 日志
```

## Makefile 命令

从项目根目录运行：

```bash
# 构建镜像
make build  # Debian 版本

# 运行服务
make run            # 生产环境（PostgreSQL）
make run-dev        # 开发环境（SQLite）
make run-mysql      # 生产环境（MySQL）

# 管理服务
make stop           # 停止所有容器
make clean          # 删除容器和卷
make logs           # 查看所有日志
make logs-server    # 仅查看服务器日志

# 测试
make test           # 在 Docker 中运行测试

# 推送到仓库
make docker-push REGISTRY=docker.io/yourusername TAG=<version>
```

## Just 命令

或者使用 `just` 命令（从项目根目录运行）：

```bash
# 构建镜像
just docker-build  # Debian 版本

# 运行服务
just docker-run            # 生产环境（PostgreSQL）
just docker-run-dev        # 开发环境（SQLite）

# 管理服务
just docker-stop           # 停止所有容器
just docker-logs           # 查看日志
```

## 健康检查

后端服务提供健康检查端点 `/ping`（返回 `{"message":"pong"}`）。

> 注意：`Dockerfile.backend` 基于 `scratch` 空镜像，不包含 curl/wget 等探针工具，因此未在镜像内配置 `HEALTHCHECK`。如需要容器级健康检查，可在编排层自行挂载探针（例如静态编译的 curl）或使用外部监控探测 `/ping`。

## 网络

所有服务运行在自定义桥接网络 `shortener-network` 中：

```yaml
networks:
  shortener-network:
    driver: bridge
```

服务可以使用服务名称相互通信：

- `shortener-server`：主应用
- `postgres`：PostgreSQL 数据库
- `mysql`：MySQL 数据库
- `redis`：Redis 缓存

## 数据持久化

使用 Docker 卷持久化数据：

- `postgres-data`：PostgreSQL 数据
- `mysql-data`：MySQL 数据
- `redis-data`：Redis 数据
- `app-logs`：应用日志
- `/app/data`：应用数据（SQLite 数据库、GeoIP 数据库等）

## 安全考虑

### 非 root 用户

容器以非 root 用户运行（`shortener`，UID 1000）：

```dockerfile
USER shortener
```

### 密钥管理

生产环境使用 Docker secrets 或环境文件：

```bash
# 创建 .env 文件
cat > .env << EOF
DATABASE__URL=postgres://shortener:your_secure_password@postgres:5432/shortener
CACHE__URL=redis://:your_redis_password@redis:6379/0
API_KEY=your_api_key
EOF

# 使用 docker compose
docker compose --env-file .env up -d
```

### 网络隔离

仅暴露必要的端口：

```yaml
ports:
  - "8080:8080" # 仅暴露服务器端口
```

生产环境不应暴露数据库和缓存端口。

## 故障排除

### 查看日志

```bash
# 所有服务
docker compose -f docker/docker-compose.yml logs -f

# 特定服务
docker compose -f docker/docker-compose.yml logs -f shortener-server
docker compose -f docker/docker-compose.yml logs -f postgres
docker compose -f docker/docker-compose.yml logs -f redis
```

### 检查容器状态

```bash
docker compose -f docker/docker-compose.yml ps
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

### 更改后重新构建

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

## 生产部署

### 1. 准备配置

```bash
# 复制并编辑配置
cp config.toml config.toml.production
vim config.toml.production
```

### 2. 设置环境变量

```bash
# 创建生产环境文件
cat > .env.production << EOF
RUST_LOG=info
DATABASE__URL=postgres://shortener:$(openssl rand -base64 32)@postgres:5432/shortener
CACHE__ENABLED=true
CACHE__URL=redis://:$(openssl rand -base64 32)@redis:6379/0
API_KEY=$(openssl rand -base64 32)
EOF
```

### 3. 部署

```bash
# 构建并启动
docker compose -f docker/docker-compose.yml --env-file .env.production up -d

# 验证
docker compose -f docker/docker-compose.yml ps
docker compose -f docker/docker-compose.yml logs -f shortener-server
```

### 4. 备份

```bash
# 备份 PostgreSQL
docker compose -f docker/docker-compose.yml exec postgres pg_dump -U shortener shortener > backup.sql

# 备份卷
docker run --rm -v shortener_postgres-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz /data
```

## 参考

- [Docker 文档](https://docs.docker.com/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [Docker Buildx Bake](https://docs.docker.com/build/bake/)
