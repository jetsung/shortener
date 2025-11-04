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
docker build -f docker/Dockerfile -t shortener-server:latest .
```

- 基础镜像：`debian:trixie-slim`
- 大小：约 150MB
- 适用于：通用场景，兼容性好

### Alpine 镜像（更小）

```bash
docker build -f docker/Dockerfile.alpine -t shortener-server:alpine .
```

- 基础镜像：`alpine:3.19`
- 大小：约 50MB
- 适用于：生产环境，最小化占用

## 使用 Docker Bake

Docker Bake 提供了更强大的构建配置。

### 本地构建

```bash
# 构建默认镜像
docker buildx bake

# 构建开发镜像（amd64）
docker buildx bake dev-amd64

# 构建开发镜像（arm64）
docker buildx bake dev-arm64

# 构建所有开发镜像
docker buildx bake dev
```

### 发布构建

```bash
# 构建多平台发布镜像
docker buildx bake release

# 这将构建 linux/amd64 和 linux/arm64 平台的镜像
```

### 自定义构建

```bash
# 使用自定义标签
docker buildx bake --set "*.tags=myregistry/shortener:v1.0.0"

# 推送到仓库
docker buildx bake --push release

# 设置平台
docker buildx bake --set "*.platform=linux/amd64,linux/arm64,linux/arm/v7"
```

## 配置

### 环境变量

可以在 `docker-compose.yml` 中设置以下环境变量：

#### 服务器配置

- `RUST_LOG`：日志级别（debug、info、warn、error）
- `CONFIG_PATH`：配置文件路径（默认：`/etc/shortener/config.toml`）

#### 数据库配置

- `DATABASE_TYPE`：数据库类型（sqlite、postgres、mysql）
- `DATABASE_HOST`：数据库主机
- `DATABASE_PORT`：数据库端口
- `DATABASE_NAME`：数据库名称
- `DATABASE_USER`：数据库用户
- `DATABASE_PASSWORD`：数据库密码
- `DATABASE_PATH`：SQLite 数据库路径

#### 缓存配置

- `CACHE_ENABLED`：启用缓存（true/false）
- `CACHE_TYPE`：缓存类型（redis、valkey）
- `CACHE_HOST`：缓存主机
- `CACHE_PORT`：缓存端口
- `CACHE_PASSWORD`：缓存密码

#### GeoIP 配置

- `GEOIP_ENABLED`：启用 GeoIP（true/false）
- `GEOIP_DB_PATH`：ip2region 数据库路径

### 卷挂载

```yaml
volumes:
  - ../config/config.toml:/etc/shortener/config.toml:ro # 配置文件
  - ../data:/var/lib/shortener:ro # 数据文件
  - shortener-logs:/var/log/shortener # 日志
```

## Makefile 命令

从项目根目录运行：

```bash
# 构建镜像
make build          # Debian 版本
make build-alpine   # Alpine 版本

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
make docker-push REGISTRY=docker.io/yourusername TAG=v1.0.0
```

## Just 命令

或者使用 `just` 命令（从项目根目录运行）：

```bash
# 构建镜像
just docker-build          # Debian 版本
just docker-build-alpine   # Alpine 版本

# 运行服务
just docker-run            # 生产环境（PostgreSQL）
just docker-run-dev        # 开发环境（SQLite）

# 管理服务
just docker-stop           # 停止所有容器
just docker-logs           # 查看日志
```

## 健康检查

服务器包含健康检查：

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 3s
  retries: 3
  start_period: 10s
```

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
- `shortener-logs`：应用日志

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
DATABASE_PASSWORD=your_secure_password
CACHE_PASSWORD=your_redis_password
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
docker compose -f docker/docker-compose.yml exec shortener-server cat /etc/shortener/config.toml

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
cp config/config.toml config/production.toml
vim config/production.toml
```

### 2. 设置环境变量

```bash
# 创建生产环境文件
cat > .env.production << EOF
RUST_LOG=info
DATABASE_TYPE=postgres
DATABASE_HOST=postgres
DATABASE_PORT=5432
DATABASE_NAME=shortener
DATABASE_USER=shortener
DATABASE_PASSWORD=$(openssl rand -base64 32)
CACHE_ENABLED=true
CACHE_TYPE=redis
CACHE_HOST=redis
CACHE_PORT=6379
CACHE_PASSWORD=$(openssl rand -base64 32)
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
