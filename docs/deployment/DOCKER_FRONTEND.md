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

前端使用 [Nginx](https://nginx.org/)（`nginx:alpine`）作为生产环境的静态文件服务器。

```bash
# 从项目根目录构建
docker build -f docker/Dockerfile.frontend -t shortener-frontend:latest .

# 运行容器
docker run -d \
  --name shortener-frontend \
  -p 80:8080 \
  shortener-frontend:latest
```

### 镜像特点

- **基础镜像**: `nginx:alpine`
- **镜像大小**: 约 60MB
- **构建方式**: 多阶段构建
- **运行用户**: 非 root 用户（nginx worker）
- **特性**:
  - 原生支持 Gzip 压缩
  - 静态资源长期缓存（Cache-Control）
  - SPA 路由回退
  - 健康检查（HEALTHCHECK）

### Dockerfile 说明

```dockerfile
# 构建阶段
FROM node:24-alpine as builder

WORKDIR /app

# 安装依赖
COPY shortener-frontend/package*.json shortener-frontend/pnpm-lock.yaml ./
RUN corepack enable && corepack prepare pnpm@latest --activate
RUN pnpm install --frozen-lockfile

# 构建应用
COPY shortener-frontend/ .
RUN pnpm build

# 生产阶段
FROM nginx:alpine

# 复制 nginx 配置（监听 8080，SPA 路由回退，gzip，静态资源缓存）
COPY docker/nginx-frontend.conf /etc/nginx/conf.d/default.conf

# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1
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
      - DATABASE__URL=sqlite:///var/lib/shortener/shortener.db?mode=rwc
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

### Nginx 配置

镜像使用 `docker/nginx-frontend.conf` 提供默认的 Nginx 配置，监听 `8080` 端口，包含：

- **SPA 路由回退**：未匹配的路径回退到 `index.html`
- **Gzip 压缩**：对文本/JS/CSS/SVG 启用
- **静态资源缓存**：`/assets/` 目录长期缓存（`immutable`），其余静态资源 30 天

如需自定义 Nginx 行为（如日志级别、缓存策略、CORS），可通过挂载自定义配置覆盖：

```yaml
services:
  shortener-frontend:
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
```

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

镜像默认已启用 Gzip 压缩（见 `docker/nginx-frontend.conf`）。如需调整，挂载自定义 Nginx 配置即可。

### 2. 启用缓存

镜像默认已对静态资源设置 `Cache-Control` 缓存头（见 `docker/nginx-frontend.conf`）。

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

镜像已内置 `HEALTHCHECK`，使用 `wget` 探测 `/` 路径。也可在编排层覆盖：

```yaml
healthcheck:
  test: ["CMD", "wget", "-q", "-O", "/dev/null", "http://127.0.0.1:8080/"]
  interval: 30s
  timeout: 3s
  retries: 3
  start_period: 5s
```

## 日志管理

### 查看日志

```bash
# 查看容器日志
# Nginx 默认输出 access/error 日志到容器 stdout/stderr
# 可通过挂载 nginx.conf 配置 access_log / error_log

# 查看容器日志
docker logs shortener-frontend

# 实时跟踪日志
docker logs -f shortener-frontend

# 查看最近 100 行日志
docker logs --tail 100 shortener-frontend

# 使用 Docker Compose
docker compose -f docker/docker-compose.frontend.yml logs -f
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

确认 SPA 回退配置正确（`docker/nginx-frontend.conf` 中的 `try_files ... /index.html`）：

```bash
# 检查 nginx 配置
nginx -t

# 确认 /usr/share/nginx/html 下有 index.html
ls /usr/share/nginx/html/index.html
```

### 调试技巧

```bash
# 1. 进入容器调试（nginx:alpine 包含 shell）
docker exec -it shortener-frontend sh

# 2. 检查构建产物
ls -la /usr/share/nginx/html

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

- [Nginx 官方文档](https://nginx.org/en/docs/)
- [Docker 最佳实践](https://docs.docker.com/develop/dev-best-practices/)
- [前端部署指南](../frontend/DEPLOYMENT.md)
