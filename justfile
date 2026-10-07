# Shortener 项目任务入口
# https://github.com/casey/just
#
# 命令按 [group('...')] 分组，just --list 按分组展示。
#
# 开发工具链由 mise 管理（根目录 mise.toml）：Rust / Node / pnpm / prek
# 进入仓库目录自动生效；若未激活 mise（mise activate / direnv 集成），
# 可用 `mise x -- just ...` 运行本文件中的命令。

set shell := ["bash", "-cu"]

# mise 执行前缀：mise 工具链就绪（cargo / pnpm 可用）时为空串，命令直接透传；
# 否则通过 mise exec 临时注入工具链环境。
mise := if `command -v cargo >/dev/null 2>&1; echo $?` == "0" { "" } else { "mise exec --" }

# 当前版本号：从 Cargo.toml (workspace.package.version) 读取
current_version := `sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1`

# 默认命令：显示帮助
default:
    @just --list

# ============================================================================
# 构建
# ============================================================================

alias b := build

# 构建全部包（后端 + 前端）
[group('构建')]
build: build-backend build-frontend

# 构建后端包
[group('构建')]
build-backend:
    {{mise}} cargo build --release

# 仅构建 server
[group('构建')]
build-server:
    {{mise}} cargo build --release -p shortener-server

# 仅构建 CLI
[group('构建')]
build-cli:
    {{mise}} cargo build --release -p shortener-cli

# 构建前端生产产物
[group('构建')]
build-frontend:
    cd shortener-frontend && {{mise}} pnpm install && pnpm build

# 构建前端并生成体积分析
[group('构建')]
build-frontend-analyze:
    cd shortener-frontend && {{mise}} pnpm install && pnpm build:analyze

# 清理构建产物
[group('构建')]
clean: clean-backend clean-frontend

# 清理后端构建产物
[group('构建')]
clean-backend:
    {{mise}} cargo clean

# 清理前端构建产物
[group('构建')]
clean-frontend:
    cd shortener-frontend && {{mise}} pnpm clean

# ============================================================================
# 运行
# ============================================================================

# 运行后端服务
[group('运行')]
run:
    {{mise}} cargo run -p shortener-server

# 运行 CLI
[group('运行')]
run-cli *ARGS:
    {{mise}} cargo run -p shortener-cli -- {{ARGS}}

# 一键本地调试：后端 :8080 + 前端 Vite dev server :8000（/api 代理到 8080）
# Ctrl-C 退出时由 trap 回收后端进程
[doc('一键本地调试：后端 + 前端 Vite（/api 代理到后端）')]
[group('运行')]
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    export PATH="$(mise exec -- printenv PATH 2>/dev/null || echo "$PATH")"

    # 后端必需密钥：未提供时使用开发默认值（对齐 acmecast 开发流程）
    export JWT_SECRET="${JWT_SECRET:-dev-only-jwt-secret-change-me}"
    if [ -z "${ADMIN__PASSWORD_HASH:-}" ]; then
        export ADMIN__PASSWORD_HASH="$(cargo run -q -p shortener-server -- hash-password -p dev -c /dev/null | head -1)"
        echo "==> 已生成开发用管理员哈希（用户 admin / 密码 dev）"
    fi

    # cargo 会再派生 shortener-server 子进程：直接 kill cargo 会把子进程变成
    # 孤儿继续占住 8080。改为 setsid 独立进程组 + 组杀（负 PID）
    setsid cargo run -p shortener-server &
    SERVER_PID=$!
    trap 'kill -- -"$SERVER_PID" 2>/dev/null || true' EXIT

    # 等待后端探活通过再启动前端；后端退出则终止并输出日志
    for _ in $(seq 1 30); do
        if curl -fsS http://127.0.0.1:8080/go/ >/dev/null 2>&1; then
            break
        fi
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then
            echo "==> 后端启动失败" >&2
            wait "$SERVER_PID" || true
            exit 1
        fi
        sleep 0.5
    done

    echo "==> 后端就绪 http://127.0.0.1:8080"
    echo "==> 启动前端 dev server（Ctrl-C 或另开终端执行 just dev-stop 停止）"
    cd shortener-frontend && {{mise}} pnpm dev

# 启动前端开发服务器
[group('运行')]
run-frontend:
    cd shortener-frontend && {{mise}} pnpm dev

# 停止本地调试环境（后端 :8080 + 前端 :8000）
[group('运行')]
dev-stop:
    #!/usr/bin/env bash
    set -uo pipefail
    stopped=0

    # 后端：shortener-server 及其 cargo 父进程（含 setsid 进程组）
    if pkill -TERM -f 'shortener-server' 2>/dev/null; then
        echo "==> 已停止后端 shortener-server"
        stopped=1
    fi
    pkill -TERM -f 'cargo run -q -p shortener-server' 2>/dev/null || true
    pkill -TERM -f 'cargo run -p shortener-server' 2>/dev/null || true

    # 前端：Vite dev server（node 进程，监听 8000）
    # 实际命令行为 node .../node_modules/.bin/../vite/bin/vite.js，
    # 以 vite/bin/vite.js 精确匹配，避免误杀路径中恰好含 "vite" 的无关进程
    if pkill -TERM -f 'vite/bin/vite\.js' 2>/dev/null; then
        echo "==> 已停止前端 Vite dev server"
        stopped=1
    fi

    if [ "$stopped" -eq 0 ]; then
        echo "==> 未发现运行中的调试进程"
    fi

# 预览前端生产构建
[group('运行')]
preview-frontend:
    cd shortener-frontend && {{mise}} pnpm preview

# ============================================================================
# 测试
# ============================================================================

alias t := test

# 运行全部测试（后端 + 前端）
[group('测试')]
test: test-backend test-frontend

# 运行后端测试
[group('测试')]
test-backend:
    {{mise}} cargo test --all

# 运行后端测试（显示输出）
[group('测试')]
test-verbose:
    {{mise}} cargo test --all -- --nocapture

# 运行基准测试
[group('测试')]
bench:
    {{mise}} cargo bench --all

# 运行前端测试
[group('测试')]
test-frontend:
    cd shortener-frontend && {{mise}} pnpm test

# 前端测试（watch 模式）
[group('测试')]
test-frontend-watch:
    cd shortener-frontend && {{mise}} pnpm test:watch

# 前端测试（覆盖率）
[group('测试')]
test-frontend-coverage:
    cd shortener-frontend && {{mise}} pnpm test:coverage

# 前端测试（UI 界面）
[group('测试')]
test-frontend-ui:
    cd shortener-frontend && {{mise}} pnpm test:ui

# ============================================================================
# 代码质量
# ============================================================================

# 格式化全部代码（后端 + 前端）
[group('代码质量')]
fmt: fmt-backend fmt-frontend

# 格式化后端代码
[group('代码质量')]
fmt-backend:
    {{mise}} cargo fmt --all

# 格式化前端代码
[group('代码质量')]
fmt-frontend:
    cd shortener-frontend && {{mise}} pnpm prettier

# 检查全部代码格式
[group('代码质量')]
fmt-check: fmt-check-backend fmt-check-frontend

# 检查后端代码格式
[group('代码质量')]
fmt-check-backend:
    {{mise}} cargo fmt --all -- --check

# 检查前端代码格式
[group('代码质量')]
fmt-check-frontend:
    cd shortener-frontend && {{mise}} pnpm prettier:check

# 运行 clippy 检查
[group('代码质量')]
clippy:
    {{mise}} cargo clippy --all-targets --all-features -- -D warnings

# 全量 Lint（后端 + 前端）
[group('代码质量')]
lint: clippy lint-frontend

# Lint 前端代码
[group('代码质量')]
lint-frontend:
    cd shortener-frontend && {{mise}} pnpm lint

# 自动修复前端 Lint 问题
[group('代码质量')]
lint-frontend-fix:
    cd shortener-frontend && {{mise}} pnpm lint:fix

# 前端类型检查
[group('代码质量')]
type-check-frontend:
    cd shortener-frontend && {{mise}} pnpm type-check

# 全量检查（后端 + 前端）
[group('代码质量')]
check: fmt-check clippy test type-check-frontend lint-frontend

# 仅检查后端
[group('代码质量')]
check-backend: fmt-check-backend clippy test-backend

# 仅检查前端
[group('代码质量')]
check-frontend: fmt-check-frontend lint-frontend type-check-frontend test-frontend

# 前端 CI 检查
[group('代码质量')]
ci-frontend:
    cd shortener-frontend && {{mise}} pnpm ci

# ============================================================================
# Docker
# ============================================================================

# 构建 Docker 镜像（统一镜像：前端 + 后端）
[group('Docker')]
docker-build:
    {{mise}} docker build -f docker/Dockerfile -t shortener:latest .

# docker compose 启动（统一镜像）
[group('Docker')]
docker-run:
    {{mise}} docker compose -f docker/docker-compose.yml up -d

# docker compose 停止全部容器
[group('Docker')]
docker-stop:
    {{mise}} docker compose -f docker/docker-compose.yml down

# 查看 Docker 日志
[group('Docker')]
docker-logs:
    {{mise}} docker compose -f docker/docker-compose.yml logs -f

# ============================================================================
# 交叉编译
# ============================================================================

# 交叉编译全部目标
[group('交叉编译')]
cross-all:
    {{mise}} ./scripts/build-cross.sh --all

# 交叉编译 server 全部目标
[group('交叉编译')]
cross-server:
    {{mise}} ./scripts/build-cross.sh --server

# 交叉编译 CLI 全部目标
[group('交叉编译')]
cross-cli:
    {{mise}} ./scripts/build-cross.sh --cli

# 交叉编译指定目标
[group('交叉编译')]
cross-target TARGET PACKAGE:
    {{mise}} ./scripts/build-cross.sh -t {{TARGET}} -p {{PACKAGE}}

# 列出可用交叉编译目标
[group('交叉编译')]
cross-list:
    {{mise}} ./scripts/build-cross.sh --list

# ============================================================================
# 发布
# ============================================================================

# 查看 Cargo.toml / openapi.yml / shortener-frontend/package.json 中的版本号
[group('发布')]
version:
    {{mise}} ./scripts/bump-version.sh

# 同步版本号到 Cargo.toml / openapi.yml / shortener-frontend/package.json
[group('发布')]
bump-version VERSION=current_version:
    {{mise}} ./scripts/bump-version.sh {{VERSION}}

# 创建新版本发布（提交 + 打 tag）
[group('发布')]
release VERSION:
    @echo "创建发布 {{VERSION}}"
    just bump-version {{VERSION}}
    git add Cargo.toml openapi.yml shortener-frontend/package.json
    git commit -m "Release {{VERSION}}"
    git tag -a "v{{VERSION}}" -m "Release {{VERSION}}"
    @echo "请执行: git push origin main --tags"

# 构建发布二进制
[group('发布')]
release-build:
    just cross-all

# ============================================================================
# 部署
# ============================================================================

# 安装 systemd 服务
[group('部署')]
install-systemd:
    cd deploy/systemd && sudo ./install.sh

# 卸载 systemd 服务
[group('部署')]
uninstall-systemd:
    cd deploy/systemd && sudo ./uninstall.sh

# ============================================================================
# 开发
# ============================================================================

# 监视变更并重新构建
[group('开发')]
watch:
    {{mise}} cargo watch -x 'run -p shortener-server'

# 监视变更并运行测试
[group('开发')]
watch-test:
    {{mise}} cargo watch -x test

# 生成 Rust 文档
[group('开发')]
doc:
    {{mise}} cargo doc --all --no-deps --open

# 更新全部依赖（后端 + 前端）
[group('开发')]
update: update-backend update-frontend

# 更新后端依赖
[group('开发')]
update-backend:
    {{mise}} cargo update

# 更新前端依赖
[group('开发')]
update-frontend:
    cd shortener-frontend && {{mise}} pnpm update

# 依赖安全审计（后端 + 前端）
[group('开发')]
audit: audit-backend

# 后端依赖安全审计
[group('开发')]
audit-backend:
    {{mise}} cargo audit

# 安装前端依赖
[group('开发')]
install-frontend:
    cd shortener-frontend && {{mise}} pnpm install

# 安装开发工具链
[group('开发')]
install-tools:
    {{mise}} cargo install cargo-watch
    {{mise}} cargo install cargo-audit
    {{mise}} cargo install cross --git https://github.com/cross-rs/cross
    {{mise}} cargo install cargo-outdated

# ============================================================================
# 文档
# ============================================================================

# 本地启动文档服务
[group('文档')]
docs:
    # 优先 uv tool install，回退 pip
    @command -v zensical >/dev/null 2>&1 || { command -v uv >/dev/null 2>&1 && uv tool install -q zensical || pip install -q zensical; }
    @echo "Starting documentation server at http://127.0.0.1:8000"
    @zensical serve

# 构建文档
[group('文档')]
docs-build:
    # 优先 uv tool install，回退 pip
    @command -v zensical >/dev/null 2>&1 || { command -v uv >/dev/null 2>&1 && uv tool install -q zensical || pip install -q zensical; }
    @echo "正在构建文档..."
    @zensical build --clean
    @echo "文档已构建到 site/"

# 部署文档到 GitHub Pages
[group('文档')]
docs-deploy:
    @echo "文档由 .github/workflows/docs.yml 在 push 到 main 时自动部署"

# ============================================================================
# 实用工具
# ============================================================================

# 显示项目统计信息
[group('实用工具')]
stats:
    @echo "=== 后端统计 ==="
    @echo "Rust 代码行数:"
    @find . -name '*.rs' -not -path './target/*' | xargs wc -l | tail -1
    @echo ""
    @echo "Rust 文件数:"
    @find . -name '*.rs' -not -path './target/*' | wc -l
    @echo ""
    @echo "后端依赖:"
    @cargo tree --depth 1
    @echo ""
    @echo "=== 前端统计 ==="
    @echo "TypeScript/TSX 代码行数:"
    @find shortener-frontend/src -name '*.ts' -o -name '*.tsx' | xargs wc -l | tail -1 || echo "N/A"
    @echo ""
    @echo "TypeScript/TSX 文件数:"
    @find shortener-frontend/src -name '*.ts' -o -name '*.tsx' | wc -l || echo "N/A"

# 检查过期依赖（后端 + 前端）
[group('实用工具')]
outdated: outdated-backend outdated-frontend

# 检查过期后端依赖
[group('实用工具')]
outdated-backend:
    {{mise}} cargo outdated

# 检查过期前端依赖
[group('实用工具')]
outdated-frontend:
    cd shortener-frontend && {{mise}} pnpm outdated

# 显示产物体积
[group('实用工具')]
sizes:
    @echo "=== 后端二进制体积 ==="
    @ls -lh target/release/shortener-server 2>/dev/null || echo "server 未构建"
    @ls -lh target/release/shortener-cli 2>/dev/null || echo "CLI 未构建"
    @echo ""
    @echo "=== 前端构建体积 ==="
    @du -sh shortener-frontend/dist 2>/dev/null || echo "前端未构建"
