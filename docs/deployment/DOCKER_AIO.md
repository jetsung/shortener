# All-In-One Docker 部署指南

本文档介绍使用 **All-In-One（AIO）镜像** 部署 shortener 服务。AIO 镜像同时包含**前端（React + nginx）**与**后端（Rust API）**，对外仅暴露一个端口，适合单机/小型部署，无需分别管理前后端两个容器。

## 目录

- [快速开始](#快速开始)
- [镜像架构](#镜像架构)
- [Docker Compose 部署](#docker-compose-部署)
- [环境变量配置](#环境变量配置)
  - [纯环境变量部署（不挂载 config.toml）](#纯环境变量部署不挂载-configtoml)
- [反向代理配置](#反向代理配置)
- [性能优化](#性能优化)
- [日志管理](#日志管理)
- [故障排查](#故障排查)
- [安全建议](#安全建议)

## 快速开始

```bash
# 1. 构建镜像（也可用 make build-aio 或 just docker-build-aio）
docker build -f docker/Dockerfile -t shortener:latest .

# 2. 使用 Docker Compose 一键启动（含 Redis）
docker compose -f docker/docker-compose.aio.yml up -d

# 3. 查看日志
docker compose -f docker/docker-compose.aio.yml logs -f

# 4. 停止
docker compose -f docker/docker-compose.aio.yml down
```

启动后访问：**http://localhost:80**

## 镜像架构

```
浏览器 ──→ :80 nginx（托管前端静态资源，前端为 hash 路由 /#/...）
              ├─ /api/* ───────────────→ 127.0.0.1:8080 shortener-server
              ├─ /ping ────────────────→ 127.0.0.1:8080（健康检查）
              ├─ /assets/*（^~ 前缀优先） → 本地静态文件（一年 immutable 缓存）
              └─ /{short_code}（字母数字） → 127.0.0.1:8080（短码重定向）
```

- **nginx**（监听 80）：负责前端静态资源托管，并将 `/api/*`、`/ping` 与短码重定向反向代理到容器内后端
- **shortener-server**（监听 `127.0.0.1:8080`）：Rust API 服务，仅容器内网可访问
- **进程管理**：`docker/entrypoint-aio.sh` 以 `setsid` 在独立会话中启动后端（信号隔离），前台运行 nginx；监控子 shell 通过 `kill -0` 轮询后端存活，后端崩溃时终止 nginx 使容器整体退出，配合 Docker `restart` 策略实现自愈

### 前端路由说明

前端采用 **hash 路由**（如 `/#/dashboard`、`/#/account/login`），`#` 及其后内容不会发送到服务端——服务端只需响应 `/`（返回 `index.html`），前端 SPA 路由不占用任何服务端路径。因此**短码路径 `/{short_code}` 与前端路由不存在冲突**，nginx 短码正则为 `^/[A-Za-z0-9]+$`（1 位及以上字母数字；实际有效短码长度由后端 `slug.length` 配置校验）。

### Dockerfile 说明

AIO 镜像为三段式多阶段构建：

| 阶段 | 基础镜像 | 产物 |
|------|---------|------|
| `builder-server` | `rust:alpine`（musl 静态链接） | `shortener-server` 静态二进制 |
| `builder-frontend` | `node:24-alpine` | 前端 Vite 构建产物 `dist` |
| 运行时 | `nginx:alpine` | 合并前端产物 + 后端二进制 |

后端以 `-c /app/config.toml` 显式指定配置路径启动（镜像 `WORKDIR /app`，配置文件即位于工作目录），并监听 `SERVER__ADDRESS=127.0.0.1:8080`（仅容器内网监听）。`config.toml` 为**可选**：镜像内置了一份默认模板，也可以不挂载文件、完全通过环境变量配置（环境变量优先于文件），详见 [环境变量配置](#环境变量配置)。

## Docker Compose 部署

项目提供现成的编排文件：`docker/docker-compose.aio.yml`。

```yaml
services:
  shortener:
    build:
      context: ..
      dockerfile: docker/Dockerfile
    container_name: shortener
    restart: unless-stopped
    ports:
      - "80:80"
    environment:
      - RUST_LOG=shortener_server=info,sqlx=off
      - DATABASE__URL=sqlite:///app/data/shortener.db?mode=rwc
      - CACHE__ENABLED=true
      - CACHE__URL=redis://redis:6379/0
      - GEOIP__ENABLED=true
      - GEOIP__IP2REGION__PATH=/app/data/ip2region.xdb
    volumes:
      - ../config.toml:/app/config.toml:ro
      - ../data:/app/data
    depends_on:
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "wget", "-q", "-O", "/dev/null", "http://127.0.0.1/ping"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s
```

> 提示：
> - 若宿主机 80 端口被占用，可改用其他端口映射，如 `"8080:80"`。
> - 生产环境务必覆盖 `api_key`、管理员密码等敏感配置——可编辑挂载的 `config.toml`，或直接用环境变量注入（推荐，见下节）。
> - `data` 目录用于 sqlite / geoip 数据，建议挂载持久化卷。
> - 不想挂载 `config.toml`？删掉该 volumes 行并补齐必填环境变量即可，见下节 [纯环境变量部署](#纯环境变量部署)。

## 环境变量配置

AIO 镜像支持通过环境变量配置/覆盖后端配置（使用 `__` 分隔嵌套键，环境变量优先于 `config.toml`）。

| 环境变量 | 必需 | 说明 |
|---------|------|------|
| `JWT_SECRET` | **是** | JWT 签名密钥，生成：`openssl rand -base64 48` |
| `SERVER__API_KEY` | **是** | API 认证密钥，生成：`openssl rand -base64 32` |
| `ADMIN__PASSWORD_HASH` | **是** | 管理员口令哈希，生成：`shortener-server hash-password --password "..."` |
| `DATABASE__URL` | **是** | 数据库连接串（`sqlite://` / `postgres://` / `mysql://`） |
| `RUST_LOG` | 否 | 日志级别（默认 `info`） |
| `SERVER__ADDRESS` | 否 | 后端监听地址（镜像默认 `127.0.0.1:8080`，勿改） |
| `ADMIN__USERNAME` | 否 | 管理员用户名（默认 `admin`） |
| `CACHE__ENABLED` / `CACHE__URL` | 否 | 缓存开关与 Redis 连接串；启用后启动时自动清空前缀旧键并从数据库预热全量短链 |
| `GEOIP__ENABLED` / `GEOIP__IP2REGION__PATH` | 否 | IP 地理定位与 ip2region 数据库路径 |

完整配置项见 [环境变量参考](../general/ENVIRONMENT_VARIABLES.md) 或 `config.toml`。

### 纯环境变量部署（不挂载 config.toml）

```yaml
services:
  shortener:
    image: ghcr.io/jetsung/shortener:latest
    container_name: shortener
    restart: unless-stopped
    ports:
      - "80:80"
    environment:
      - RUST_LOG=shortener_server=info,sqlx=off
      - JWT_SECRET=${JWT_SECRET}
      - SERVER__API_KEY=${SERVER__API_KEY}
      - ADMIN__PASSWORD_HASH=${ADMIN__PASSWORD_HASH}
      - DATABASE__URL=sqlite:///app/data/shortener.db?mode=rwc
      - CACHE__ENABLED=true
      - CACHE__URL=redis://redis:6379/0
    volumes:
      - ./data:/app/data
```

配合 `.env` 文件（与 compose 同目录，`docker compose` 自动读取）：

```bash
# .env
JWT_SECRET=...                  # openssl rand -base64 48
SERVER__API_KEY=...             # openssl rand -base64 32
ADMIN__PASSWORD_HASH=$argon2id$...   # shortener-server hash-password --password "..."
```

> 其余配置（监听地址、短码长度、管理员用户名等）均有合理默认值，必填项缺失时启动会给出明确的字段级错误。

## 反向代理配置

AIO 镜像本身已用 nginx 承担静态托管与反代，外部无需再配置后端反代。若需在 AIO 之前再套一层网关（HTTPS 终结、负载均衡），将 80 端口作为上游即可。

**协议透传**：AIO 内置 nginx 会透传上游的 `X-Forwarded-Proto` 与 `X-Real-IP`（未携带时回退本机值），后端据此推导 OIDC 回调地址（`redirect_uri`）与真实客户端 IP。因此外层网关务必设置：

```nginx
proxy_set_header X-Forwarded-Proto $scheme;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header Host $http_host;
```

否则 HTTPS 部署下 OIDC 回调地址会被推导为 `http://`，IdP 校验 Redirect URI 将失败。

### 使用 Caddy

```caddyfile
short.example.com {
    reverse_proxy shortener:80
}
```

### 使用 Traefik

```yaml
services:
  traefik:
    image: traefik:v3
    ...
  shortener:
    build:
      context: ..
      dockerfile: docker/Dockerfile
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.aio.rule=Host(`short.example.com`)"
      - "traefik.http.routers.aio.entrypoints=websecure"
      - "traefik.http.routers.aio.tls=true"
      - "traefik.http.services.aio.loadbalancer.server.port=80"
```

## 性能优化

### 1. 压缩与缓存

nginx 默认已启用 Gzip 压缩，并对 `/assets/`（Vite 带内容哈希产物）设置长期 `immutable` 缓存、其余静态资源 30 天缓存（见 `docker/nginx-aio.conf`）。

### 2. 资源限制

```yaml
services:
  shortener:
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## 日志管理

```bash
# 查看容器日志
docker logs shortener

# 实时跟踪
docker logs -f shortener

# 使用 Docker Compose
docker compose -f docker/docker-compose.aio.yml logs -f
```

- **后端日志**：由 `RUST_LOG` 控制，输出到容器 stdout
- **nginx 日志**：access/error 日志默认输出到容器 stdout/stderr

## 故障排查

### 1. 容器无法启动

```bash
# 检查后端能否单独启动
docker run --rm -it --entrypoint /usr/local/bin/shortener-server shortener:latest --help

# 检查 nginx 配置
docker exec shortener nginx -t
```

### 2. 前端白屏 / API 请求失败

```bash
# 检查 nginx 反代是否正常
docker exec shortener wget -q -O- http://127.0.0.1/ping

# 检查后端日志
docker logs shortener 2>&1 | grep -i error
```

### 3. 短码无法访问

- 确认短码为纯字母数字（nginx 短码正则为 `location ~ "^/[A-Za-z0-9]+$"`），长度在配置的 `slug.length` 生成规则内
- 检查后端是否正常（`docker exec shortener wget -q -O- http://127.0.0.1/ping`）

### 4. 数据目录权限

AIO 镜像后端以容器默认用户运行，挂载的 `data` 卷需保证可写：

```bash
chown -R 65532:65532 ./data   # 若后端以非 root 运行
# 或检查挂载卷权限
docker exec shortener ls -la /app/data
```

## CI/CD 发布

AIO 镜像通过 GitHub Actions 自动构建与发布，相关 workflow：

### 发布镜像（`docker-release-aio.yml`）

- **触发**：推送 `shortener-server-v*` tag（与后端发布共用同一 tag，打 `shortener-server-vX.Y.Z` 时 server 与 AIO 镜像同步构建）
- **产物**：多架构（amd64 + arm64）OCI 归档
- **推送目标**：
  - Docker Hub：`jetsung/shortener`（`:latest` / `:${VERSION}`）
  - GHCR：`ghcr.io/jetsung/shortener`
  - 阿里云 ACR / 腾讯云 TCR（配置凭证后自动同步，否则跳过）

### 开发镜像（`docker-dev-aio.yml`）

- **触发**：推送 `dev*` 分支，且改动涉及 AIO 相关文件（`docker/Dockerfile`、`nginx-aio.conf`、`entrypoint-aio.sh`、后端源码、前端源码、`config.toml` 等）
- **产物**：多架构（amd64 + arm64）
- **推送目标**：Docker Hub `jetsung/shortener`，合并 manifest 后输出 `:dev` 与 `:latest`

> 说明：workflow 内 git tag 前缀剥离使用 `shortener-server-v`，与触发 tag 一致；镜像名统一为 `shortener`，不区分 `-server`/`-frontend` 后缀。

## 安全建议

1. **修改默认凭据**：部署前务必修改 `config.toml` 中的 `api_key`、admin 密码等默认值
2. **使用 HTTPS**：在外部网关（Caddy/Traefik/云负载均衡）终结 TLS
3. **限制访问**：配置防火墙规则，必要时启用 rate limiting
4. **定期更新**：及时更新基础镜像（`nginx:alpine`、`rust:alpine`、`node:24-alpine`）与依赖

## 相关文档

- [Docker 部署（后端）](DOCKER.md)
- [前端 Docker 部署](DOCKER_FRONTEND.md)
- [Docker 高级部署](DOCKER_ADVANCED.md)
