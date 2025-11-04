# 部署指南

本文档详细介绍了如何将 Shortener Frontend 部署到不同的环境中。

## 目录

- [构建准备](#构建准备)
- [环境变量配置](#环境变量配置)
- [Docker 部署](#docker-部署)
- [Nginx 部署](#nginx-部署)
- [云服务部署](#云服务部署)
- [性能优化](#性能优化)
- [监控和日志](#监控和日志)

## 构建准备

### 系统要求

- Node.js >= 18.0.0
- pnpm >= 8.0.0 (推荐) 或 npm >= 9.0.0
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

### 测试环境 (.env.test)

```bash
# API 基础地址
VITE_API_BASE_URL=https://test-api.yourdomain.com

# 应用标题
VITE_APP_TITLE=Shortener (测试)

# 是否启用调试模式
VITE_DEBUG=true

# 是否启用 Mock 数据
VITE_USE_MOCK=false
```

## Docker 部署

### 方式一：多阶段构建

创建 `Dockerfile`：

```dockerfile
# 构建阶段
FROM node:18-alpine as builder

# 设置工作目录
WORKDIR /app

# 复制 package 文件
COPY package*.json pnpm-lock.yaml ./

# 安装 pnpm
RUN npm install -g pnpm

# 安装依赖
RUN pnpm install --frozen-lockfile

# 复制源代码
COPY . .

# 构建应用
RUN pnpm build

# 生产阶段
FROM nginx:alpine

# 复制构建结果
COPY --from=builder /app/dist /usr/share/nginx/html

# 复制 nginx 配置
COPY nginx.conf /etc/nginx/nginx.conf

# 暴露端口
EXPOSE 80

# 启动 nginx
CMD ["nginx", "-g", "daemon off;"]
```

### 方式二：使用预构建镜像

创建 `docker-compose.yml`：

```yaml
version: '3.8'

services:
  shortener-frontend:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "80:80"
    environment:
      - VITE_API_BASE_URL=https://api.yourdomain.com
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    restart: unless-stopped
    networks:
      - shortener-network

networks:
  shortener-network:
    driver: bridge
```

### 构建和运行

```bash
# 构建镜像
docker build -t shortener-frontend .

# 运行容器
docker run -d \
  --name shortener-frontend \
  -p 80:80 \
  -e VITE_API_BASE_URL=https://api.yourdomain.com \
  shortener-frontend

# 或使用 docker-compose
docker-compose up -d
```

## Nginx 部署

### Nginx 配置文件

创建 `nginx.conf`：

```nginx
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # 日志格式
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    # 基本设置
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    client_max_body_size 16M;

    # Gzip 压缩
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/json
        application/javascript
        application/xml+rss
        application/atom+xml
        image/svg+xml;

    # 缓存设置
    map $sent_http_content_type $expires {
        default                    off;
        text/html                  epoch;
        text/css                   max;
        application/javascript     max;
        ~image/                    max;
        ~font/                     max;
    }

    server {
        listen 80;
        server_name localhost;
        root /usr/share/nginx/html;
        index index.html;

        # 设置缓存
        expires $expires;

        # 处理 SPA 路由
        location / {
            try_files $uri $uri/ /index.html;
        }

        # API 代理 (可选)
        location /api/ {
            proxy_pass http://backend-server:8080/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # 静态资源缓存
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }

        # 安全头
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-XSS-Protection "1; mode=block" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header Referrer-Policy "no-referrer-when-downgrade" always;
        add_header Content-Security-Policy "default-src 'self' http: https: data: blob: 'unsafe-inline'" always;

        # 健康检查
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
}
```

### 部署步骤

1. **上传构建文件**

```bash
# 将 dist 目录内容上传到服务器
scp -r dist/* user@server:/var/www/shortener/

# 或使用 rsync
rsync -avz --delete dist/ user@server:/var/www/shortener/
```

2. **配置 Nginx**

```bash
# 复制配置文件
sudo cp nginx.conf /etc/nginx/sites-available/shortener
sudo ln -s /etc/nginx/sites-available/shortener /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx
```

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
  NODE_VERSION = "18"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200

[[headers]]
  for = "/static/*"
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

### 1. 启用 Gzip/Brotli 压缩

```nginx
# Nginx 配置
gzip on;
gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;

# 或启用 Brotli (需要模块支持)
brotli on;
brotli_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
```

### 2. 设置合适的缓存策略

```nginx
# 静态资源长期缓存
location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# HTML 文件不缓存
location ~* \.html$ {
    expires -1;
    add_header Cache-Control "no-cache, no-store, must-revalidate";
}
```

### 3. 启用 HTTP/2

```nginx
server {
    listen 443 ssl http2;
    # SSL 配置...
}
```

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

### 3. 访问日志分析

使用 Nginx 日志分析工具：

```bash
# 安装 GoAccess
sudo apt-get install goaccess

# 分析日志
goaccess /var/log/nginx/access.log -o report.html --log-format=COMBINED
```

### 4. 健康检查

```nginx
# Nginx 健康检查端点
location /health {
    access_log off;
    return 200 "healthy\n";
    add_header Content-Type text/plain;
}
```

## 故障排除

### 常见问题

1. **白屏问题**
   - 检查控制台错误
   - 确认静态资源路径正确
   - 检查 API 地址配置

2. **路由 404 错误**
   - 确认 Nginx 配置了 SPA 路由重写
   - 检查 `try_files` 配置

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
# 检查 Nginx 配置
sudo nginx -t

# 查看 Nginx 日志
sudo tail -f /var/log/nginx/error.log

# 检查端口占用
sudo netstat -tlnp | grep :80

# 检查服务状态
sudo systemctl status nginx
```

## 安全建议

1. **HTTPS 配置**
   - 使用 Let's Encrypt 免费证书
   - 配置 HSTS 头
   - 禁用不安全的 SSL/TLS 版本

2. **安全头设置**
   ```nginx
   add_header X-Frame-Options "SAMEORIGIN";
   add_header X-XSS-Protection "1; mode=block";
   add_header X-Content-Type-Options "nosniff";
   add_header Referrer-Policy "strict-origin-when-cross-origin";
   ```

3. **访问控制**
   - 配置防火墙规则
   - 限制访问频率
   - 使用 CDN 防护

4. **定期更新**
   - 及时更新依赖包
   - 定期更新服务器系统
   - 监控安全漏洞

更多部署相关问题，请参考 [Semi Design 部署指南](https://semi.design/zh-CN/start/getting-started#%E9%83%A8%E7%BD%B2) 或联系技术支持。
