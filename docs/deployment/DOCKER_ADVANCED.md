# Docker 部署指南

本文档介绍如何使用 Docker 和 Docker Compose 部署 Shortener URL 短链接服务。

## 目录

- [快速开始](#快速开始)
- [Docker Compose 部署](#docker-compose-部署)
- [单独使用 Docker](#单独使用-docker)
- [数据库配置](#数据库配置)
- [环境变量](#环境变量)
- [数据持久化](#数据持久化)
- [健康检查](#健康检查)
- [故障排查](#故障排查)

## 快速开始

使用 Docker Compose 快速启动完整的服务栈（包括 PostgreSQL 和 Redis）：

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 使用 Docker Compose 启动服务
docker compose -f docker/docker-compose.yml up -d

# 查看日志
docker compose -f docker/docker-compose.yml logs -f shortener-server

# 停止服务
docker compose -f docker/docker-compose.yml down
```

服务启动后，访问 http://localhost:8080 即可使用。

## Docker Compose 部署

### 完整服务栈（推荐）

使用 [`docker-compose.yml`](docker-compose.yml) 部署包含 PostgreSQL 和 Redis 的完整服务：

```bash
docker compose -f docker/docker-compose.yml up -d
```

该配置包含：
- **shortener-server**: URL 短链接服务（端口 8080）
- **postgres**: PostgreSQL 数据库（端口 5432）
- **redis**: Redis 缓存（端口 6379）

### 使用 SQLite（轻量级部署）

如果不需要 PostgreSQL，可以使用 SQLite：

```yaml
services:
  shortener-server:
    image: jetsung/shortener-server:latest
    container_name: shortener-server
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_TYPE=sqlite
      - DATABASE_PATH=/var/lib/shortener/shortener.db
      - CACHE_ENABLED=false
      - GEOIP_ENABLED=true
    volumes:
      - ./config/config.toml:/etc/shortener/config.toml:ro
      - ./data:/var/lib/shortener
      - shortener-logs:/var/log/shortener

volumes:
  shortener-logs:
```

### 使用 MySQL

如果需要使用 MySQL 而不是 PostgreSQL：

```bash
# 启动 MySQL 配置
docker compose -f docker/docker-compose.yml --profile mysql up -d
```

或修改 `docker-compose.yml` 中的环境变量：

```yaml
environment:
  - DATABASE_TYPE=mysql
  - DATABASE_HOST=mysql
  - DATABASE_PORT=3306
  - DATABASE_NAME=shortener
  - DATABASE_USER=shortener
  - DATABASE_PASSWORD=shortener_password
```

### 开发环境

使用 [`docker-compose.dev.yml`](docker-compose.dev.yml) 进行开发：

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

开发环境配置包含：
- 热重载支持
- 详细的日志输出
- 开发工具集成

## 单独使用 Docker

### 构建镜像

```bash
# 从项目根目录构建
docker build -f docker/Dockerfile -t shortener-server:latest .

# 使用 Alpine 基础镜像（更小的镜像体积）
docker build -f docker/Dockerfile.alpine -t shortener-server:alpine .
```

### 运行容器

```bash
# 使用 SQLite（最简单）
docker run -d \
  --name shortener-server \
  -p 8080:8080 \
  -v $(pwd)/config/config.toml:/etc/shortener/config.toml:ro \
  -v $(pwd)/data:/var/lib/shortener \
  -e RUST_LOG=info \
  -e DATABASE_TYPE=sqlite \
  shortener-server:latest

# 连接到外部 PostgreSQL
docker run -d \
  --name shortener-server \
  -p 8080:8080 \
  -v $(pwd)/config/config.toml:/etc/shortener/config.toml:ro \
  -e RUST_LOG=info \
  -e DATABASE_TYPE=postgres \
  -e DATABASE_HOST=your-postgres-host \
  -e DATABASE_PORT=5432 \
  -e DATABASE_NAME=shortener \
  -e DATABASE_USER=shortener \
  -e DATABASE_PASSWORD=your-password \
  shortener-server:latest
```

## 数据库配置

### PostgreSQL（推荐用于生产环境）

```yaml
services:
  shortener-server:
    environment:
      - DATABASE_TYPE=postgres
      - DATABASE_HOST=postgres
      - DATABASE_PORT=5432
      - DATABASE_NAME=shortener
      - DATABASE_USER=shortener
      - DATABASE_PASSWORD=shortener_password
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_DB=shortener
      - POSTGRES_USER=shortener
      - POSTGRES_PASSWORD=shortener_password
    volumes:
      - postgres-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U shortener"]
      interval: 10s
      timeout: 5s
      retries: 5
```

### MySQL

```yaml
services:
  shortener-server:
    environment:
      - DATABASE_TYPE=mysql
      - DATABASE_HOST=mysql
      - DATABASE_PORT=3306
      - DATABASE_NAME=shortener
      - DATABASE_USER=shortener
      - DATABASE_PASSWORD=shortener_password
    depends_on:
      mysql:
        condition: service_healthy

  mysql:
    image: mysql:8.0
    environment:
      - MYSQL_ROOT_PASSWORD=root_password
      - MYSQL_DATABASE=shortener
      - MYSQL_USER=shortener
      - MYSQL_PASSWORD=shortener_password
    volumes:
      - mysql-data:/var/lib/mysql
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 10s
      timeout: 5s
      retries: 5
```

### SQLite（适合小规模部署）

```yaml
services:
  shortener-server:
    environment:
      - DATABASE_TYPE=sqlite
      - DATABASE_PATH=/var/lib/shortener/shortener.db
    volumes:
      - ./data:/var/lib/shortener
```

## 环境变量

### 服务器配置

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `RUST_LOG` | 日志级别 | `info` |
| `CONFIG_PATH` | 配置文件路径 | `/etc/shortener/config.toml` |
| `SERVER_ADDRESS` | 监听地址 | `0.0.0.0:8080` |
| `SITE_URL` | 站点 URL | `http://localhost:8080` |

### 数据库配置

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `DATABASE_TYPE` | 数据库类型（sqlite/postgres/mysql） | `postgres` |
| `DATABASE_HOST` | 数据库主机 | `localhost` |
| `DATABASE_PORT` | 数据库端口 | `5432` |
| `DATABASE_NAME` | 数据库名称 | `shortener` |
| `DATABASE_USER` | 数据库用户 | `shortener` |
| `DATABASE_PASSWORD` | 数据库密码 | - |
| `DATABASE_PATH` | SQLite 数据库路径 | `/var/lib/shortener/shortener.db` |

### 缓存配置

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `CACHE_ENABLED` | 是否启用缓存 | `true` |
| `CACHE_TYPE` | 缓存类型（redis/valkey） | `redis` |
| `CACHE_HOST` | 缓存主机 | `localhost` |
| `CACHE_PORT` | 缓存端口 | `6379` |
| `CACHE_PASSWORD` | 缓存密码 | - |
| `CACHE_EXPIRE` | 缓存过期时间（秒） | `3600` |

### GeoIP 配置

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `GEOIP_ENABLED` | 是否启用 GeoIP | `true` |
| `GEOIP_DB_PATH` | GeoIP 数据库路径 | `/var/lib/shortener/ip2region.xdb` |

## 数据持久化

Docker Compose 配置使用以下卷来持久化数据：

```yaml
volumes:
  postgres-data:      # PostgreSQL 数据
  mysql-data:         # MySQL 数据
  redis-data:         # Redis 数据
  shortener-logs:     # 应用日志
```

### 备份数据

```bash
# 备份 PostgreSQL
docker exec shortener-postgres pg_dump -U shortener shortener > backup.sql

# 备份 SQLite
docker cp shortener-server:/var/lib/shortener/shortener.db ./backup.db

# 备份配置
cp config/config.toml config/config.toml.backup
```

### 恢复数据

```bash
# 恢复 PostgreSQL
docker exec -i shortener-postgres psql -U shortener shortener < backup.sql

# 恢复 SQLite
docker cp ./backup.db shortener-server:/var/lib/shortener/shortener.db
```

## 健康检查

服务包含健康检查端点：

```bash
# 检查服务健康状态
curl http://localhost:8080/health

# 查看 Docker 健康状态
docker ps
docker inspect shortener-server | grep -A 10 Health
```

## 故障排查

### 查看日志

```bash
# 查看所有服务日志
docker compose -f docker/docker-compose.yml logs

# 查看特定服务日志
docker compose -f docker/docker-compose.yml logs shortener-server
docker compose -f docker/docker-compose.yml logs postgres
docker compose -f docker/docker-compose.yml logs redis

# 实时跟踪日志
docker compose -f docker/docker-compose.yml logs -f shortener-server
```

### 常见问题

#### 1. 数据库连接失败

检查数据库服务是否正常运行：

```bash
# 检查 PostgreSQL
docker exec shortener-postgres pg_isready -U shortener

# 检查 MySQL
docker exec shortener-mysql mysqladmin ping -h localhost
```

#### 2. 端口冲突

如果端口 8080 已被占用，修改 `docker-compose.yml` 中的端口映射：

```yaml
ports:
  - "8081:8080"  # 使用 8081 端口
```

#### 3. 权限问题

确保数据目录有正确的权限：

```bash
sudo chown -R 1000:1000 data/
sudo chmod -R 755 data/
```

#### 4. 容器无法启动

检查容器日志和配置：

```bash
docker logs shortener-server
docker inspect shortener-server
```

### 重置服务

```bash
# 停止并删除所有容器和卷
docker compose -f docker/docker-compose.yml down -v

# 重新启动
docker compose -f docker/docker-compose.yml up -d
```

## 生产环境建议

1. **使用环境变量文件**: 创建 `.env` 文件管理敏感信息
2. **启用 HTTPS**: 使用 Nginx 或 Traefik 作为反向代理
3. **配置资源限制**: 在 `docker-compose.yml` 中添加资源限制
4. **定期备份**: 设置自动备份任务
5. **监控和日志**: 集成 Prometheus、Grafana 等监控工具
6. **使用 PostgreSQL**: 生产环境推荐使用 PostgreSQL 而不是 SQLite
7. **启用 Redis**: 启用缓存以提升性能

### 资源限制示例

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

## 相关文档

- [主文档](../README.md)
- [配置指南](../docs/CONFIGURATION.md)
- [API 文档](../docs/API.md)
- [部署指南](../docs/DEPLOYMENT.md)
