# Docker 完整部署指南

本文档介绍如何使用 Docker 端到端部署完整的 Shortener 服务栈（后端 API + 前端管理界面 + 数据库 + 缓存 + 反向代理）。

> 基础后端部署（单服务、环境变量）请参阅 [Docker 部署指南](DOCKER.md)；
> 高级主题（多平台构建、性能调优、安全加固）请参阅 [Docker 高级部署指南](DOCKER_ADVANCED.md)。

## 目录

- [服务架构](#服务架构)
- [快速开始（完整栈）](#快速开始完整栈)
- [后端服务](#后端服务)
- [前端服务](#前端服务)
- [数据库与缓存](#数据库与缓存)
- [反向代理](#反向代理)
- [数据持久化](#数据持久化)
- [备份与恢复](#备份与恢复)

## 服务架构

一个完整的生产部署通常包含：

- **shortener-server**：Rust 后端 API（端口 8080）
- **shortener-frontend**：React 前端（Nginx，端口 8080）
- **postgres**：PostgreSQL 数据库（可选，生产推荐）
- **redis**：Redis/Valkey 缓存
- **Caddy / Nginx / Traefik**：反向代理与 HTTPS

## 快速开始（完整栈）

使用 Docker Compose 启动包含 PostgreSQL 和 Redis 的完整后端：

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 使用 Docker Compose 启动
docker compose -f docker/docker-compose.yml up -d

# 查看日志
docker compose -f docker/docker-compose.yml logs -f shortener-server

# 停止服务
docker compose -f docker/docker-compose.yml down
```

服务启动后，访问 http://localhost:8080 即可使用。

## 后端服务

### 构建镜像

```bash
# 标准镜像（Debian）
docker build -f docker/Dockerfile.backend -t shortener-server:latest .
```

### 运行容器（使用 SQLite）

```bash
docker run -d \
  --name shortener-server \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/data:/app/data \
  -e RUST_LOG=info \
  -e DATABASE__URL=sqlite:///app/data/shortener.db?mode=rwc \
  shortener-server:latest
```

### 运行容器（连接外部 PostgreSQL）

```bash
docker run -d \
  --name shortener-server \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -e RUST_LOG=info \
  -e DATABASE__URL=postgres://shortener:your-password@your-postgres-host:5432/shortener \
  shortener-server:latest
```

## 前端服务

前端使用 [Nginx](https://nginx.org/)（`nginx:alpine`）作为生产环境的静态文件服务器。

```bash
# 构建前端镜像
docker build -f docker/Dockerfile.frontend -t shortener-frontend .

# 运行容器
docker run -d \
  --name shortener-frontend \
  -p 80:8080 \
  shortener-frontend
```

前端详细部署说明见 [前端 Docker 部署指南](DOCKER_FRONTEND.md)。

## 数据库与缓存

### PostgreSQL（生产推荐）

```yaml
services:
  shortener-server:
    environment:
      - DATABASE__URL=postgres://shortener:shortener_password@postgres:5432/shortener
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
      - DATABASE__URL=mysql://shortener:shortener_password@mysql:3306/shortener

  mysql:
    image: mysql:8.0
    environment:
      - MYSQL_ROOT_PASSWORD=root_password
      - MYSQL_DATABASE=shortener
      - MYSQL_USER=shortener
      - MYSQL_PASSWORD=shortener_password
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
```

> `docker-compose.yml` 中 MySQL 服务默认带 `profiles: [mysql]`，需显式启用：`docker compose --profile mysql up -d`。

### Redis / Valkey 缓存

```yaml
services:
  shortener-server:
    environment:
      - CACHE__ENABLED=true
      - CACHE__URL=redis://:redis_password@redis:6379/0

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --requirepass redis_password
    healthcheck:
      test: ["CMD", "redis-cli", "--raw", "incr", "ping"]
```

## 反向代理

### Nginx

```nginx
upstream shortener {
    server localhost:8080;
}

server {
    listen 80;
    server_name short.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name short.example.com;

    ssl_certificate /etc/letsencrypt/live/short.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/short.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    location / {
        proxy_pass http://shortener;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

```caddyfile
short.example.com {
    reverse_proxy localhost:8080
}
```

> 反向代理需正确转发 OIDC 回调路径 `https://<域名>/api/oidc/callback` 到本服务。

## 数据持久化

Docker Compose 配置使用以下卷持久化数据：

```yaml
volumes:
  postgres-data:      # PostgreSQL 数据
  mysql-data:         # MySQL 数据
  redis-data:         # Redis 数据
  app-logs:           # 应用日志
```

## 备份与恢复

```bash
# 备份 PostgreSQL
docker compose -f docker/docker-compose.yml exec postgres pg_dump -U shortener shortener > backup.sql

# 备份 SQLite
docker cp shortener-server:/app/data/shortener.db ./backup.db

# 备份配置
cp config.toml config.toml.backup
```

恢复：

```bash
# 恢复 PostgreSQL
docker exec -i shortener-postgres psql -U shortener shortener < backup.sql

# 恢复 SQLite
docker cp ./backup.db shortener-server:/app/data/shortener.db
```

## 生产环境建议

1. **使用环境变量文件**：创建 `.env` 管理敏感信息，`docker compose --env-file .env up -d`
2. **启用 HTTPS**：使用 Caddy / Nginx / Traefik 作为反向代理
3. **配置资源限制**：见 [Docker 高级部署指南](DOCKER_ADVANCED.md)
4. **定期备份**：设置自动备份任务
5. **使用 PostgreSQL**：生产环境推荐使用 PostgreSQL 而非 SQLite
6. **启用 Redis**：开启缓存以提升性能

## 相关文档

- [Docker 部署指南](DOCKER.md)
- [Docker 高级部署指南](DOCKER_ADVANCED.md)
- [前端 Docker 部署指南](DOCKER_FRONTEND.md)
- [配置指南](../general/CONFIGURATION.md)
- [API 文档](../server/API.md)