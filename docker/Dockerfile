# Shortener 统一镜像 Dockerfile（对齐 acmecast 形态）
# 单一进程：shortener-server 直接托管前端静态产物并对外提供 API，
# 无 nginx、无进程管理脚本——单二进制即完整服务。
#
# 运行时架构：
#   - shortener-server（监听 0.0.0.0:8080）：
#     · /            → 前端静态文件（SERVER__STATIC_DIR=/static，未命中回退 index.html）
#     · /api/*       → 业务 API（含 /api/ping 健康检查）
#     · /go/{code}   → 短码跳转
#
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

# 构建 release（musl 目标完全静态，distroless static 可直接运行）
ENV OPENSSL_STATIC=1
RUN cargo build --release -p shortener-server --bins

# 预创建数据目录并归属 nonroot(65532)：distroless 无 shell，不能在运行时阶段 mkdir
RUN mkdir -p /app/data && chown 65532:65532 /app/data

# ============================================================================
# Stage 2: 构建前端（Vite 产物）
# ============================================================================
FROM node:24-alpine AS builder-frontend

WORKDIR /app

# 复制 package 文件（利用层缓存）
COPY shortener-frontend/package.json shortener-frontend/pnpm-lock.yaml ./

# 安装 pnpm（版本由 package.json 的 packageManager 字段锁定，保证构建可复现）
RUN corepack enable

# 安装依赖
RUN pnpm install --frozen-lockfile

# 复制源代码并构建
COPY shortener-frontend/ ./
RUN pnpm build

# ============================================================================
# Stage 3: 运行时镜像（distroless static，非 root，对齐 acmecast）
# ============================================================================
# 最终成品运行目录统一为 /app（WORKDIR），因此配置中的相对路径直接以 /app 为基准：
#   - DATABASE__URL=sqlite://./data/shortener.db?mode=rwc  → /app/data/shortener.db
#   - GEOIP__IP2REGION__PATH=./data/ip2region.xdb          → /app/data/ip2region.xdb
# 目录布局：
#   /app/config.toml  配置文件
#   /app/data/        数据目录（SQLite / GeoIP）
#   /app/static/      前端静态产物
FROM gcr.io/distroless/static-debian13:nonroot

# WORKDIR /app：使 DATABASE__URL / GEOIP__IP2REGION__PATH 等相对路径配置
# （./data/...）无需改写即可工作，与本地二进制部署的行为一致
WORKDIR /app

# 后端静态二进制
COPY --from=builder-server /app/target/release/shortener-server /usr/bin/shortener-server

# 后端配置文件（可选：完全可用环境变量配置，挂载卷可覆盖）
COPY config.toml /app/config.toml

# 前端构建产物（由后端静态托管，未命中回退 index.html）
COPY --from=builder-frontend /app/dist /app/static

# 数据目录（SQLite / GeoIP）挂卷持久化（目录已在 builder 阶段预创建并归属 nonroot）
COPY --chown=65532:65532 --from=builder-server /app/data /app/data

# 环境变量
# 单进程直接对外，无需反代：监听 0.0.0.0:8080；静态产物路径与 WORKDIR 对齐为 /app/static
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config.toml
ENV SERVER__ADDRESS=0.0.0.0:8080
ENV SERVER__STATIC_DIR=/app/static

VOLUME ["/app/data"]

# 唯一对外端口
EXPOSE 8080

# 启动入口
ENTRYPOINT ["/usr/bin/shortener-server"]
