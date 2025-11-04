# 前端 Docker 部署指南

本文档介绍如何使用 Docker 部署 Shortener Frontend。

## 目录

- [快速开始](#快速开始)
- [Docker 镜像](#docker-镜像)
- [Docker Compose 部署](#docker-compose-部署)
- [环境变量配置](#环境变量配置)
- [反向代理配置](#反向代理配置)
- [故障排查](#故障排查)

## 快速开始

### 使用 Docker Compose

```bash
# 启动前端服务
docker compose -f docker/docker-compose.frontend.yml up -d

# 查看日志
docker compose -f docker/docker-compose.frontend.yml logs -f

# 停止服务
docker compose -f docker/docker-compose.frontend.yml down
```

服务启动后，访问 http://localhost 即可使用前端界面。

## Docker 镜像

### 构建镜像

前端使用 [static-web-server](https://static-web-server.net/) 作为生产环境的静态文件服务器。

```bash
# 从项目根目录构建
docker build -f docker/Dockerfile.frontend -t shortener-frontend:latest ./shortener-frontend

# 运行容器
docker run -d \
  --name shortener-frontend \
  -p 80:8080 \
  -e SERVER_LOG_LEVEL=info \
  shortener-frontend:latest
```

### 镜像特点

- **基础镜像**: `joseluisq/static-web-server:2`
- **镜像大小**: 约 10MB（极小）
- **构建方式**: 多阶段构建
- **运行用户**: 非 root 用户
- **特性**: 
  - 原生支持 Gzip 和 Brotli 压缩
  - 自动缓存控制
  - SPA 路由支持
  - HTTP/2 支持

### Dockerfile 说明

```dockerfile
# 构建阶段
FROM node:20-alpine as builder

WORKDIR /app

# 安装依赖
COPY package*.json pnpm-lock.yaml ./
RUN corepack enable && corepack prepare pnpm@10.18.3 --activate
RUN pnpm install --frozen-lockfile

# 构建应用
COPY . .
RUN pnpm build

# 生产阶段
FROM joseluisq/static-web-server:2

# 复制构建产物
COPY --from=builder /app/dist /public

# 配置环境变量
ENV SERVER_PORT=8080 \
    SERVER_ROOT=/public \
    SERVER_LOG_LEVEL=info \
    SERVER_FALLBACK_PAGE=/public/index.html \
    SERVER_COMPRESSION_GZIP=true \
    SERVER_COMPRESSION_BROTLI=true \
    SERVER_CACHE_CONTROL_HEADERS=true

EXPOSE 8080
```

## Docker Compose 部署

### 基础配置

`docker/docker-compose.frontend.yml`:

```yaml
version: '3.8'

services:
  shortener-frontend:
    build:
      context: ../shortener-frontend
      dockerfile: ../docker/Dockerfile.frontend
    container_name: shortener-frontend
    ports:
      - "80:8080"
    environment:
      - SERVER_PORT=8080
      - SERVER_ROOT=/public
      - SERVER_LOG_LEVEL=info
      - SERVER_FALLBACK_PAGE=/public/index.html
      - SERVER_COMPRESSION_GZIP=true
      - SERVER_COMPRESSION_BROTLI=true
      - SERVER_CACHE_CONTROL_HEADERS=true
    restart: unless-stopped
    networks:
      - shortener-network

networks:
  shortener-network:
    driver: bridge
    name: shortener-network
```

### 与后端服务集成

创建完整的服务栈配置：

```yaml
version: '3.8'

services:
  # 后端服务
  shortener-server:
    image: ghcr.io/jetsung/shortener-server:latest
    container_name: shortener-server
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_TYPE=sqlite
      - DATABASE_PATH=/var/lib/shortener/shortener.db
    volumes:
      - ./data:/var/lib/shortener
    networks:
      - shortener-network
    restart: unless-stopped

  # 前端服务
  shortener-frontend:
    build:
      context: ../shortener-frontend
      dockerfile: ../docker/Dockerfile.frontend
    container_name: shortener-frontend
    ports:
      - "80:8080"
    environment:
      - SERVER_PORT=8080
      - SERVER_ROOT=/public
      - SERVER_LOG_LEVEL=info
      - SERVER_FALLBACK_PAGE=/public/index.html
      - SERVER_COMPRESSION_GZIP=true
      - SERVER_COMPRESSION_BROTLI=true
    depends_on:
      - shortener-server
    networks:
      - shortener-network
    restart: unless-stopped

networks:
  shortener-network:
    driver: bridge
    name: shortener-network
```

## 环境变量配置

### Static Web Server 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `SERVER_PORT` | 8080 | 服务监听端口 |
| `SERVER_ROOT` | /public | 静态文件根目录 |
| `SERVER_HOST` | 0.0.0.0 | 监听地址 |
| `SERVER_LOG_LEVEL` | error | 日志级别 (error/warn/info/debug/trace) |
| `SERVER_FALLBACK_PAGE` | - | SPA 路由回退页面（设置为 /public/index.html） |
| `SERVER_COMPRESSION_GZIP` | false | 启用 Gzip 压缩 |
| `SERVER_COMPRESSION_BROTLI` | false | 启用 Brotli 压缩 |
| `SERVER_CACHE_CONTROL_HEADERS` | false | 启用缓存控制头 |
| `SERVER_CORS_ALLOW_ORIGINS` | - | CORS 允许的源（逗号分隔） |
| `SERVER_CORS_ALLOW_HEADERS` | - | CORS 允许的请求头 |

### 构建时环境变量

在构建阶段可以通过 `.env` 文件配置 Vite 环境变量：

```bash
# .env.production
VITE_API_BASE_URL=https://api.yourdomain.com
VITE_APP_TITLE=Shortener
VITE_DEBUG=false
```

然后在构建时使用：

```bash
docker build \
  --build-arg VITE_API_BASE_URL=https://api.yourdomain.com \
  -f docker/Dockerfile.frontend \
  -t shortener-frontend:latest \
  ./shortener-frontend
```

## 反向代理配置

### 使用 Caddy

`Caddyfile`:

```caddy
short.example.com {
    reverse_proxy shortener-frontend:8080
    
    # 安全头
    header {
        X-Frame-Options "SAMEORIGIN"
        X-XSS-Protection "1; mode=block"
        X-Content-Type-Options "nosniff"
        Referrer-Policy "strict-origin-when-cross-origin"
    }
    
    # 启用压缩
    encode gzip zstd
    
    # 日志
    log {
        output file /var/log/caddy/shortener.log
    }
}
```

### 使用 Traefik

`docker-compose.yml`:

```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v2.10
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
    ports:
      - "80:80"
      - "443:443"
      - "8081:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    networks:
      - shortener-network

  shortener-frontend:
    build:
      context: ../shortener-frontend
      dockerfile: ../docker/Dockerfile.frontend
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.frontend.rule=Host(`short.example.com`)"
      - "traefik.http.routers.frontend.entrypoints=websecure"
      - "traefik.http.routers.frontend.tls=true"
      - "traefik.http.services.frontend.loadbalancer.server.port=8080"
    networks:
      - shortener-network

networks:
  shortener-network:
    driver: bridge
```

## 性能优化

### 1. 启用压缩

```yaml
environment:
  - SERVER_COMPRESSION_GZIP=true
  - SERVER_COMPRESSION_BROTLI=true
```

### 2. 启用缓存

```yaml
environment:
  - SERVER_CACHE_CONTROL_HEADERS=true
```

### 3. 资源限制

```yaml
services:
  shortener-frontend:
    # ... 其他配置
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
        reservations:
          cpus: '0.25'
          memory: 128M
```

### 4. 健康检查

由于 static-web-server:2 基于 scratch 构建，无法在容器内执行健康检查命令。建议在编排层配置：

```yaml
# 使用外部健康检查
healthcheck:
  test: ["CMD-SHELL", "wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

或者使用 Traefik/Kubernetes 的健康检查功能。

## 日志管理

### 查看日志

```bash
# 查看容器日志
docker logs shortener-frontend

# 实时跟踪日志
docker logs -f shortener-frontend

# 查看最近 100 行日志
docker logs --tail 100 shortener-frontend

# 使用 Docker Compose
docker compose -f docker/docker-compose.frontend.yml logs -f
```

### 日志级别

通过环境变量控制日志详细程度：

```yaml
environment:
  # 生产环境：只记录错误
  - SERVER_LOG_LEVEL=error
  
  # 开发环境：记录详细信息
  - SERVER_LOG_LEVEL=debug
```

## 故障排查

### 常见问题

#### 1. 容器无法启动

```bash
# 查看容器状态
docker ps -a | grep shortener-frontend

# 查看详细日志
docker logs shortener-frontend

# 检查端口占用
sudo netstat -tlnp | grep :8080
```

#### 2. 页面无法访问

```bash
# 检查容器是否运行
docker ps | grep shortener-frontend

# 检查端口映射
docker port shortener-frontend

# 测试容器内部服务
docker exec shortener-frontend wget -O- http://localhost:8080
```

#### 3. API 请求失败

检查前端构建时的 API 地址配置：

```bash
# 查看构建时的环境变量
docker inspect shortener-frontend | grep VITE_API_BASE_URL
```

#### 4. 路由 404 错误

确认设置了 SPA 回退页面：

```yaml
environment:
  - SERVER_FALLBACK_PAGE=/public/index.html
```

### 调试技巧

```bash
# 1. 使用 alpine 版本进行调试（包含 shell）
docker run -it joseluisq/static-web-server:2-alpine sh

# 2. 检查构建产物
docker run --rm -it --entrypoint sh shortener-frontend:latest
ls -la /public

# 3. 测试静态文件服务
curl -I http://localhost:8080/

# 4. 检查压缩是否生效
curl -H "Accept-Encoding: gzip" -I http://localhost:8080/
```

## 安全建议

1. **使用 HTTPS**
   - 在反向代理层配置 SSL/TLS
   - 使用 Let's Encrypt 免费证书

2. **设置安全头**
   - 在反向代理层添加安全响应头
   - 配置 CSP（Content Security Policy）

3. **限制访问**
   - 配置防火墙规则
   - 使用 rate limiting
   - 启用 CORS 白名单

4. **定期更新**
   - 及时更新基础镜像
   - 更新前端依赖包
   - 监控安全漏洞

## 参考资源

- [Static Web Server 官方文档](https://static-web-server.net/)
- [Static Web Server 环境变量](https://static-web-server.net/configuration/environment-variables/)
- [Docker 最佳实践](https://docs.docker.com/develop/dev-best-practices/)
- [前端部署指南](../frontend/DEPLOYMENT.md)
