# 部署指南

本文档详细介绍了如何将 Shortener Frontend 部署到不同的环境中。

## 目录

- [构建准备](#构建准备)
- [环境变量配置](#环境变量配置)
- [Docker 部署](#docker-部署)
- [云服务部署](#云服务部署)
- [性能优化](#性能优化)
- [监控和日志](#监控和日志)

## 构建准备

### 系统要求

- Node.js >= 18.0.0
- pnpm >= 10.0.0 (推荐) 或 npm >= 9.0.0
- 至少 2GB 可用内存用于构建

### 构建步骤

1. **安装依赖**

```bash
pnpm install --frozen-lockfile
```

2. **类型检查**

```bash
pnpm type-check
```

3. **代码质量检查**

```bash
pnpm lint
```

4. **运行测试**

```bash
pnpm test
```

5. **生产构建**

```bash
pnpm build
```

构建完成后，所有静态文件将生成在 `dist/` 目录中。

## 环境变量配置

### 开发环境 (.env.development)

```bash
# API 基础地址
VITE_API_BASE_URL=http://localhost:8080

# 应用标题
VITE_APP_TITLE=Shortener (开发)

# 是否启用调试模式
VITE_DEBUG=true

# 是否启用 Mock 数据
VITE_USE_MOCK=false
```

### 生产环境 (.env.production)

```bash
# API 基础地址
VITE_API_BASE_URL=https://api.yourdomain.com

# 应用标题
VITE_APP_TITLE=Shortener

# 是否启用调试模式
VITE_DEBUG=false

# 是否启用 Mock 数据
VITE_USE_MOCK=false

# CDN 地址 (可选)
VITE_CDN_URL=https://cdn.yourdomain.com

# 应用版本
VITE_APP_VERSION=2.0.0
```

## Docker 部署

### 使用 Nginx

我们使用 [Nginx](https://nginx.org/)（`nginx:alpine`）作为生产环境的静态文件服务器。

创建 `Dockerfile`：

```dockerfile
# 构建阶段
FROM node:24-alpine as builder

WORKDIR /app

COPY shortener-frontend/package*.json shortener-frontend/pnpm-lock.yaml ./

RUN corepack enable && corepack prepare pnpm@latest --activate

RUN pnpm install --frozen-lockfile

COPY shortener-frontend/ .

RUN pnpm build

# 生产阶段
FROM nginx:alpine

# 复制 nginx 配置（监听 8080，SPA 路由回退，gzip，静态资源缓存）
COPY docker/nginx-frontend.conf /etc/nginx/conf.d/default.conf

COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1
```

### Docker Compose 部署

创建 `docker-compose.yml`：

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

### 构建和运行

```bash
# 构建镜像
docker build -f docker/Dockerfile.frontend -t shortener-frontend .

# 运行容器
docker run -d \
  --name shortener-frontend \
  -p 80:8080 \
  shortener-frontend

# 或使用 docker compose
docker compose -f docker/docker-compose.frontend.yml up -d
```

### Nginx 配置

镜像使用 `docker/nginx-frontend.conf` 提供默认的 Nginx 配置，监听 `8080` 端口，包含：

- **SPA 路由回退**：未匹配的路径回退到 `index.html`
- **Gzip 压缩**：对文本/JS/CSS/SVG 启用
- **静态资源缓存**：`/assets/` 目录长期缓存（`immutable`），其余静态资源 30 天

如需自定义，可挂载自定义 Nginx 配置覆盖 `/etc/nginx/conf.d/default.conf`。

## 云服务部署

### Vercel 部署

1. **安装 Vercel CLI**

```bash
npm i -g vercel
```

2. **配置 vercel.json**

```json
{
  "version": 2,
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/static-build",
      "config": {
        "distDir": "dist"
      }
    }
  ],
  "routes": [
    {
      "handle": "filesystem"
    },
    {
      "src": "/(.*)",
      "dest": "/index.html"
    }
  ],
  "env": {
    "VITE_API_BASE_URL": "@api_base_url"
  }
}
```

3. **部署**

```bash
vercel --prod
```

### Netlify 部署

1. **配置 netlify.toml**

```toml
[build]
  publish = "dist"
  command = "pnpm build"

[build.environment]
  NODE_VERSION = "20"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200

[[headers]]
  for = "/assets/*"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
```

2. **部署**

```bash
# 安装 Netlify CLI
npm install -g netlify-cli

# 部署
netlify deploy --prod --dir=dist
```

### AWS S3 + CloudFront 部署

1. **构建并上传到 S3**

```bash
# 构建
pnpm build

# 上传到 S3
aws s3 sync dist/ s3://your-bucket-name --delete

# 设置缓存策略
aws s3 cp dist/index.html s3://your-bucket-name/index.html \
  --cache-control "no-cache"
```

2. **配置 CloudFront**

创建 CloudFront 分发，配置：
- Origin: S3 bucket
- Default Root Object: index.html
- Error Pages: 404 -> /index.html (for SPA routing)
- Caching: 根据文件类型设置不同的缓存策略

## 性能优化

### 1. 压缩配置

镜像内置的 Nginx 配置默认启用 Gzip 压缩（见 `docker/nginx-frontend.conf`）。如需调整，挂载自定义 Nginx 配置：

```yaml
volumes:
  - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
```

### 2. 缓存策略

镜像默认对静态资源设置缓存头（`/assets/` 长期缓存 `immutable`，其余 30 天）。

### 3. HTTP/2 支持

Nginx 原生支持 HTTP/2，在反向代理或直接暴露时配置即可。

### 4. 预加载关键资源

在 `index.html` 中添加：

```html
<link rel="preload" href="/assets/main.js" as="script">
<link rel="preload" href="/assets/main.css" as="style">
```

## 监控和日志

### 1. 应用监控

集成应用性能监控工具：

```typescript
// 在 main.tsx 中添加
import { initWebVitals } from './utils/performance';

// 初始化性能监控
initWebVitals();
```

### 2. 错误监控

```typescript
// 全局错误处理
window.addEventListener('error', (event) => {
  console.error('Global error:', event.error);
  // 发送到错误监控服务
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason);
  // 发送到错误监控服务
});
```

### 3. 日志级别

Nginx 默认输出 access/error 日志到容器 stdout/stderr。如需调整日志级别，挂载自定义 Nginx 配置：

```nginx
error_log /var/log/nginx/error.log warn;
```

## 故障排除

### 常见问题

1. **白屏问题**
   - 检查控制台错误
   - 确认静态资源路径正确
   - 检查 API 地址配置

2. **路由 404 错误**
   - 确认 SPA 回退配置正确（`try_files ... /index.html`）
   - 检查 SPA 路由配置

3. **API 请求失败**
   - 检查 CORS 配置
   - 确认 API 地址正确
   - 检查网络连接

4. **构建失败**
   - 检查 Node.js 版本
   - 清理 node_modules 重新安装
   - 检查内存是否足够

### 调试工具

```bash
# 查看容器日志
docker logs shortener-frontend

# 进入容器调试（nginx:alpine 包含 shell）
docker exec -it shortener-frontend sh

# 检查端口占用
sudo netstat -tlnp | grep :8080

# 检查容器状态
docker ps -a | grep shortener-frontend
```

## 安全建议

1. **HTTPS 配置**
   - 使用反向代理（如 Caddy、Traefik）处理 HTTPS
   - 配置 HSTS 头
   - 禁用不安全的 SSL/TLS 版本

2. **安全头设置**

   使用反向代理添加安全头：

   ```nginx
   # Caddy 示例
   header {
       X-Frame-Options "SAMEORIGIN"
       X-XSS-Protection "1; mode=block"
       X-Content-Type-Options "nosniff"
       Referrer-Policy "strict-origin-when-cross-origin"
   }
   ```

3. **访问控制**
   - 配置防火墙规则
   - 限制访问频率
   - 使用 CDN 防护

4. **定期更新**
   - 及时更新依赖包
   - 定期更新容器镜像
   - 监控安全漏洞

更多部署相关问题，请参考 [Nginx 文档](https://nginx.org/en/docs/) 或联系技术支持。
