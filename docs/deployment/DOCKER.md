# Docker 部署指南

本指南说明如何使用 Docker 和 Docker Compose 构建与部署 Shortener。

> 项目已收敛为**单一 Dockerfile**（`docker/Dockerfile`）：一个镜像同时包含
> 前端（nginx）与后端（shortener-server），不再提供分离的前端/后端镜像。

## 快速开始

### 构建镜像

```bash
docker build -f docker/Dockerfile -t shortener:latest .

# 或使用 Just
just docker-build
```

### 启动服务（SQLite + Redis）

```bash
# 构建并运行
docker compose -f docker/docker-compose.yml up -d

# 查看日志
docker compose -f docker/docker-compose.yml logs -f

# 停止
docker compose -f docker/docker-compose.yml down
```

服务启动后访问 `http://localhost/`（nginx 监听 80）。

### 使用 PostgreSQL / MySQL（可选）

默认使用 SQLite（`./data` 目录）。需要外部数据库时启用对应 profile：

```bash
# PostgreSQL
docker compose -f docker/docker-compose.yml --profile postgres up -d
# 并把 shortener 服务的 DATABASE__URL 改为
# postgres://shortener:shortener_password@postgres:5432/shortener

# MySQL
docker compose -f docker/docker-compose.yml --profile mysql up -d
```

## 镜像架构

统一镜像内包含两个进程，由 `docker/entrypoint-aio.sh` 拉起：

- **nginx**（监听 80）：托管前端静态资源（前端为 hash 路由 `/#/...`），并反向代理 `/api/*` 与 `/go/` 短码重定向到容器内后端
- **shortener-server**（监听 `127.0.0.1:8080`）：API 服务，仅容器内网可访问；后端以 `setsid` 独立会话运行，nginx 前台运行

## 使用 Docker Bake

Docker Bake 提供了更强大的构建配置。

### 本地构建

```bash
# 构建默认镜像
docker buildx bake -f docker/docker-bake.hcl

# 构建开发镜像（amd64 / arm64 / 全部）
docker buildx bake -f docker/docker-bake.hcl dev-amd64
docker buildx bake -f docker/docker-bake.hcl dev-arm64
docker buildx bake -f docker/docker-bake.hcl dev
```

### 发布构建

```bash
# 构建多平台发布镜像（linux/amd64 + linux/arm64）
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

## 配置

### 环境变量

可以在 `docker-compose.yml` 中设置以下环境变量。

#### 服务器配置

- `RUST_LOG`：日志级别（debug、info、warn、error）

其余服务器配置项使用双下划线 `__` 分隔嵌套键（由 `config` crate 自动映射）：

- `SERVER__SHORT_URL`：短址专用域名（可选，未设置时从监听地址推断，通配地址回退 localhost）
- `SERVER__API_KEY`：API 密钥

> `SERVER__ADDRESS` 已由镜像固定为 `127.0.0.1:8080`（后端仅容器内网监听，由 nginx 对外代理），无需覆盖。

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
  - ../data:/app/data # 数据文件（SQLite、GeoIP 等）
```

## Just 命令

从项目根目录运行：

```bash
# 构建镜像
just docker-build

# 运行 / 停止 / 日志
just docker-run
just docker-stop
just docker-logs
```

## 健康检查

镜像内置 `HEALTHCHECK`：通过 nginx 探测后端 `/api/ping`（返回 `{"message":"pong"}`）。

## 网络

所有服务运行在自定义桥接网络 `shortener-network` 中：

```yaml
networks:
  shortener-network:
    driver: bridge
```

服务可以使用服务名称相互通信：

- `shortener`：统一镜像（nginx + 后端）
- `postgres`：PostgreSQL 数据库（postgres profile）
- `mysql`：MySQL 数据库（mysql profile）
- `redis`：Redis 缓存

## 数据持久化

使用 Docker 卷持久化数据：

- `postgres-data`：PostgreSQL 数据（postgres profile）
- `mysql-data`：MySQL 数据（mysql profile）
- `redis-data`：Redis 数据
- `../data`：应用数据（SQLite 数据库、GeoIP 数据库等，宿主机目录挂载）

## 安全考虑

### 密钥管理

生产环境使用 Docker secrets 或环境文件：

```bash
# 创建 .env 文件
cat > .env << EOF
SERVER__API_KEY=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 48)
CACHE__URL=redis://redis:6379/0
EOF

# 使用 docker compose
docker compose --env-file .env up -d
```

### 网络隔离

仅暴露必要的端口：

```yaml
ports:
  - "80:80" # 仅暴露 nginx 端口
```

生产环境不应暴露数据库和缓存端口。

## 故障排除

### 查看日志

```bash
# 所有服务
docker compose -f docker/docker-compose.yml logs -f

# 特定服务
docker compose -f docker/docker-compose.yml logs -f shortener
docker compose -f docker/docker-compose.yml logs -f redis
```

### 检查容器状态

```bash
docker compose -f docker/docker-compose.yml ps
```

### 在容器中执行命令

```bash
# Shell 访问
docker compose -f docker/docker-compose.yml exec shortener sh

# 检查配置
docker compose -f docker/docker-compose.yml exec shortener cat /app/config.toml
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
SERVER__API_KEY=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 48)
CACHE__ENABLED=true
CACHE__URL=redis://redis:6379/0
EOF
```

### 3. 部署

```bash
# 构建并启动
docker compose -f docker/docker-compose.yml --env-file .env.production up -d

# 验证
docker compose -f docker/docker-compose.yml ps
docker compose -f docker/docker-compose.yml logs -f shortener
```

### 4. 备份

```bash
# 备份 SQLite 数据（宿主机 ./data 目录）
tar czf backup.tar.gz data/

# 备份 Redis 卷
docker run --rm -v shortener_redis-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/redis-backup.tar.gz /data
```

## 参考

- [Docker 文档](https://docs.docker.com/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [Docker Buildx Bake](https://docs.docker.com/build/bake/)
- [All-In-One 部署详解](DOCKER_AIO.md)
