# All-In-One Dockerfile: 同时打包 shortener-server（后端）与 shortener-frontend（前端）为单个镜像
#
# 镜像内进程架构：
#   - nginx（监听 80）：托管前端静态资源 + SPA 回退，并反向代理 /api/* 与短码重定向到后端
#   - shortener-server（监听 127.0.0.1:8080）：API 服务（仅容器内网，由 nginx 代理对外）
# 两个进程由 docker/entrypoint-aio.sh 同时拉起。

# ============================================================================
# Stage 1: 构建后端（静态链接 musl 二进制）
# ============================================================================
ARG RUST_VERSION=1.98
FROM rust:$RUST_VERSION-alpine AS builder-server

# 安装构建依赖（musl 静态链接 + OpenSSL 静态库）
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

WORKDIR /app

# 复制清单文件
COPY Cargo.toml Cargo.lock ./
COPY shortener-server/Cargo.toml ./shortener-server/
COPY shortener-cli/Cargo.toml ./shortener-cli/
COPY shortener-common/Cargo.toml ./shortener-common/

# 复制源码
COPY shortener-server/src ./shortener-server/src
COPY shortener-cli/src ./shortener-cli/src
COPY shortener-common/src ./shortener-common/src

# 复制 benches（Cargo.toml 校验需要，但不会编译）
COPY shortener-server/benches ./shortener-server/benches

# 构建 release（musl 目标默认完全静态，强制 OpenSSL 静态链接）
ENV OPENSSL_STATIC=1
RUN cargo build --release -p shortener-server --bins

# ============================================================================
# Stage 2: 构建前端（Vite 产物）
# ============================================================================
FROM node:24-alpine AS builder-frontend

WORKDIR /app

# 复制 package 文件
COPY shortener-frontend/package*.json shortener-frontend/pnpm-lock.yaml ./

# 安装 pnpm
RUN corepack enable && corepack prepare pnpm@latest --activate

# 安装依赖
RUN pnpm install --frozen-lockfile

# 复制源代码
COPY shortener-frontend/ .

# 构建应用
RUN pnpm build

# ============================================================================
# Stage 3: 运行时镜像（nginx:alpine，内置 shell 用于进程管理与健康检查）
# ============================================================================
FROM nginx:alpine

# 工作目录：后端以此为基础解析相对路径（如默认的 config.toml、data/）
WORKDIR /app

# 后端静态二进制
COPY --from=builder-server /app/target/release/shortener-server /usr/local/bin/shortener-server

# 后端配置文件（挂载卷可覆盖）
COPY config.toml /app/config.toml

# 前端构建产物
COPY --from=builder-frontend /app/dist /usr/share/nginx/html

# nginx AIO 配置（静态托管 + SPA 回退 + /api 与短码反代）
COPY docker/nginx-aio.conf /etc/nginx/conf.d/default.conf

# 进程管理入口脚本（后台启动后端 + 前台运行 nginx）
COPY docker/entrypoint-aio.sh /usr/local/bin/entrypoint-aio.sh
RUN chmod +x /usr/local/bin/entrypoint-aio.sh

# 预创建数据目录（sqlite / geoip）
RUN mkdir -p /app/data && chown nginx:nginx /app/data

# 环境变量
# 后端仅监听容器内网，由 nginx 对外代理（127.0.0.1:8080）
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config.toml
ENV SERVER__ADDRESS=127.0.0.1:8080

# 暴露端口（nginx 对外端口）
EXPOSE 80

# 健康检查（探测 nginx 反代到后端的 /ping）
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget -q -O /dev/null http://127.0.0.1/ping || exit 1

# 启动入口
ENTRYPOINT ["/usr/local/bin/entrypoint-aio.sh"]
