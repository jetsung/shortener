# 统一镜像部署指南

本文档介绍使用 **Shortener 统一镜像** 部署服务。镜像内是**单一进程**
`shortener-server`（对齐 acmecast 形态）：直接托管前端静态产物并对外提供
API，无 nginx、无进程管理脚本，适合单机/小型部署，无需分别管理前后端两个容器。

## 快速开始

```bash
# 1. 构建镜像（也可用 just docker-build）
docker build -f docker/Dockerfile -t shortener:latest .

# 2. 使用 Docker Compose 一键启动（含 Redis）
docker compose -f docker/docker-compose.yml up -d

# 3. 查看日志
docker compose -f docker/docker-compose.yml logs -f

# 4. 停止
docker compose -f docker/docker-compose.yml down
```

启动后访问：**http://localhost:8080**

## 镜像架构

```
浏览器 ──→ :8080 shortener-server（单进程，distroless static 运行时、非 root）
              ├─ /              → 前端静态文件（/static，未命中回退 index.html）
              ├─ /assets/*      → Vite 带内容哈希产物
              ├─ /api/*         → 业务 API（含 /api/ping 健康检查）
              └─ /go/{short_code} → 短码跳转
```

- **单进程**：`shortener-server` 静态链接二进制，直接托管静态资源与 API，
  无 nginx、无 entrypoint 脚本；容器崩溃自愈完全由 Docker `restart` 策略承担
- **非 root**：distroless `nonroot` 用户（UID 65532），数据目录已预授权
- **数据持久化**：`/app/data`（SQLite / GeoIP），挂卷即可

### 前端路由说明

前端采用 **hash 路由**（如 `/#/dashboard`、`/#/account/login`），`#` 及其后
内容不会发送到服务端——服务端只需响应 `/`（返回 `index.html`）。因此**短码
路径 `/go/{short_code}` 与前端路由不存在冲突**（短码统一走 `/go/` 前缀；
实际有效短码长度由后端 `slug.length` 配置校验）。

### 旧短链兼容（可选）

v0.3.0 之前短链形态历经 `/{short_code}` 与 `/to/{short_code}` 两代，现统一为
`/go/{short_code}`。已对外分发的旧短链可在**外层** nginx（TLS 终结层）加一条
301 重定向过渡：

```nginx
# 放在外层 server 块中，置于其它 location 之前
rewrite ^/([A-Za-z0-9]+)$ /go/$1 permanent;
```

## Dockerfile 说明

统一镜像为三段式多阶段构建：

| 阶段 | 基础镜像 | 产出 |
| --- | --- | --- |
| builder-server | `rust:1.98-alpine` | musl 静态链接的 `shortener-server` 二进制 + `/app/data` 目录 |
| builder-frontend | `node:24-alpine` | Vite 前端产物（pnpm 版本由 package.json 锁定） |
| 运行时 | `gcr.io/distroless/static-debian13:nonroot` | 仅二进制 + 静态产物 + 配置 |

环境变量（镜像内置默认值）：

| 变量 | 值 | 说明 |
| --- | --- | --- |
| `SERVER__ADDRESS` | `0.0.0.0:8080` | 单进程直接对外 |
| `SERVER__STATIC_DIR` | `/static` | 前端静态产物目录（未命中回退 `index.html`） |
| `CONFIG_PATH` | `/app/config.toml` | 配置文件路径（可挂载覆盖） |

## 环境变量配置

镜像支持通过环境变量覆盖后端配置（使用 `__` 分隔嵌套键，环境变量优先于
`config.toml`）。完整清单见 [环境变量参考](../general/ENVIRONMENT_VARIABLES.md)。

常用项：

```bash
SERVER__API_KEY=<openssl rand -base64 32>   # API 密钥（必填）
JWT_SECRET=<openssl rand -base64 48>        # JWT 签名密钥
ADMIN__PASSWORD_HASH='$argon2id$...'        # 管理员口令哈希
DATABASE__URL=sqlite:///app/data/shortener.db?mode=rwc
CACHE__ENABLED=true
CACHE__URL=redis://redis:6379/0
GEOIP__ENABLED=true
GEOIP__IP2REGION__PATH=/app/data/ip2region.xdb
```

## Docker Compose 部署

项目提供现成的编排文件：`docker/docker-compose.yml`（shortener + Redis，
PostgreSQL/MySQL 通过 profile 可选启用）。

```bash
# 默认（SQLite + Redis）
docker compose -f docker/docker-compose.yml up -d

# 启用 PostgreSQL
docker compose -f docker/docker-compose.yml --profile postgres up -d
# 并把 shortener 服务的 DATABASE__URL 改为
# postgres://shortener:shortener_password@postgres:5432/shortener
```

## 外部 Nginx 反向代理

单进程直接对外，后端**不需要**任何反代即可工作（`http://服务器IP:8080`）。
需要 HTTPS 终结、域名接入或负载均衡时，在外层加一台 nginx，把 `8080` 作为上游，
**任意路径原样转发，无需任何改写**。

### 推荐完整配置

一个主域名（控制台 + API + 短码）+ 一个专用短域名（仅短码，链接无 `/go/` 前缀）的典型配置：

```nginx
# ---------- 公共上游与透传 ----------
upstream shortener {
    server 127.0.0.1:8080;
    keepalive 32;
}

# ---------- 主域名：控制台 + API + 短码 ----------
server {
    listen 80;
    server_name short.example.com;
    # 全站跳 HTTPS（首次用 HTTP 申请证书时临时注释这两行）
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    http2 on;
    server_name short.example.com;

    ssl_certificate     /etc/letsencrypt/live/short.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/short.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    # 上传/请求体限制（按需调整）
    client_max_body_size 10m;

    location / {
        proxy_pass http://shortener;
        proxy_http_version 1.1;

        # 必设透传头（见下方「透传头说明」）
        proxy_set_header Host              $http_host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket（前端 dev 热更新或未来实时功能）
        proxy_set_header Upgrade    $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        # 长连接复用
        proxy_set_header Connection "";
    }
}

# WebSocket 升级映射（放在 http{} 层，与 upstream 同级）
# map $http_upgrade $connection_upgrade {
#     default upgrade;
#     ""      close;
# }

# ---------- 专用短域名：仅短码（链接形如 https://s.example.com/<code>） ----------
# 使用此配置时，服务端配置 SERVER__SHORT_URL=https://s.example.com
server {
    listen 443 ssl;
    http2 on;
    server_name s.example.com;

    ssl_certificate     /etc/letsencrypt/live/s.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/s.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    # 旧版短链（/{code} 与 /to/{code}）301 过渡到 /go/{code}
    rewrite ^/([A-Za-z0-9]+)$          /go/$1 permanent;
    rewrite ^/to/([A-Za-z0-9]+)$       /go/$1 permanent;

    location /go/ {
        proxy_pass http://shortener;
        proxy_http_version 1.1;
        proxy_set_header Host              $http_host;
        proxy_set_header X-Real-IP         $remote_addr;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # 短址域名只服务跳转，其余路径一律 404
    location / {
        return 404;
    }
}
```

> `map $http_upgrade $connection_upgrade` 必须位于 `http {}` 层（`upstream` 同级），
> 不能放进 `server {}`；若不需要 WebSocket，可删除对应两行 `proxy_set_header`。

### 透传头说明（必设）

后端依赖以下请求头推导**真实客户端 IP**（访问统计/GeoIP）与 **OIDC 回调地址**，
缺失或错误会导致统计数据失真、OIDC 登录失败：

| 请求头 | 设置值 | 后端用途 |
| --- | --- | --- |
| `Host` | `$http_host` | OIDC `redirect_uri` 推导（`{scheme}://{host}/api/oidc/callback`） |
| `X-Real-IP` | `$remote_addr` | 真实客户端 IP |
| `X-Forwarded-For` | `$proxy_add_x_forwarded_for` | 真实客户端 IP（链式） |
| `X-Forwarded-Proto` | `$scheme` | OIDC `redirect_uri` 的 scheme 部分 |

多层代理（CDN → nginx → 容器）时，`X-Real-IP` 使用最外层传来的值即可；
若外层还有 CDN，请以 `real_ip` 模块或 CDN 文档为准取最原始客户端 IP。

### 可选：外层缓存与压缩

后端已对 `/assets/`（Vite 内容哈希产物）返回长期 `Cache-Control` 头、页面回退
`index.html`，通常无需额外配置。如需进一步降低回源量，可在主域名 server 块追加：

```nginx
gzip on;
gzip_comp_level 5;
gzip_min_length 1024;
gzip_types text/css application/javascript application/json image/svg+xml;

# 静态哈希资源直接命中外层缓存（可选）
location /assets/ {
    proxy_pass http://shortener;
    proxy_cache_valid 200 30d;
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

### 其他网关

Caddy：

```caddyfile
short.example.com {
    reverse_proxy 127.0.0.1:8080
}
# 专用短域名同理：
# s.example.com { reverse_proxy 127.0.0.1:8080 }
```

Traefik：

```yaml
labels:
  - "traefik.http.routers.shortener.rule=Host(`short.example.com`)"
  - "traefik.http.routers.shortener.entrypoints=websecure"
  - "traefik.http.routers.shortener.tls=true"
  - "traefik.http.services.shortener.loadbalancer.server.port=8080"
```

## 性能说明

静态资源缓存由后端处理：`/assets/`（Vite 带内容哈希产物）返回长期缓存头，
页面与深链回退 `index.html`。如需更激进的压缩/缓存策略，可在外层网关配置。

## 日志

- 后端日志：由 `RUST_LOG` 控制，输出到容器 stdout（JSON 格式）

```bash
# 实时跟踪
docker logs -f shortener
```

## 故障排查

### 页面无法访问

- 确认容器在运行：`docker compose -f docker/docker-compose.yml ps`
- 检查后端日志：`docker logs shortener`

### 健康检查

```bash
# 容器内直接探测（宿主机映射端口同理）
curl http://127.0.0.1:8080/api/ping   # {"message":"pong"}
```

> 镜像为 distroless（无 shell/wget），未内置容器级 `HEALTHCHECK` 探针；
> compose 的 healthcheck 已禁用。健康监控建议使用外部探针请求 `/api/ping`。

### 短码无法访问

- 确认访问路径使用 `/go/` 前缀（如 `/go/abc123`），短码为纯字母数字且长度在
  配置的 `slug.length` 生成规则内；旧形态 `/{short_code}` 需外层 nginx 301
  重定向（见「旧短链兼容」）

### 数据目录权限

镜像以非 root（UID 65532）运行，挂载的 `data` 卷需保证可写：

```bash
mkdir -p data && sudo chown -R 65532:65532 data
```

## CI/CD

镜像通过 GitHub Actions 自动构建与发布：

- **开发镜像**（`docker-dev-aio.yml`）：推送 `dev*` 分支时构建
  `ghcr.io/jetsung/shortener:dev`（触发路径含 `docker/Dockerfile`、后端/前端源码）
- **发布镜像**（`docker-release-aio.yml`）：推送 `shortener-server-v*` tag 时
  构建多平台（amd64/arm64）发布镜像并同步到 Docker Hub / GHCR / 阿里云 / 华为云 / 腾讯云

## 安全建议

1. **修改默认凭证**：部署前设置 `SERVER__API_KEY`、`JWT_SECRET`、
   `ADMIN__PASSWORD_HASH`（勿使用示例值）
2. **最小暴露**：仅映射 `8080`，数据库/Redis 端口不对宿主机暴露
3. **及时更新**：升级基础镜像（`rust:alpine`、`node:24-alpine`、distroless）与依赖

## 相关文档

- [Docker 部署（快速开始）](DOCKER.md)
- [Docker 高级部署](DOCKER_ADVANCED.md)
