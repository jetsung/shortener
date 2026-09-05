#!/bin/bash

# 文档构建脚本
# 用于构建 Zensical 文档

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查是否安装了 Zensical（优先 uv，回退 pip）
if ! command -v zensical &> /dev/null; then
    print_info "Zensical 未安装，正在安装..."
    if command -v uv &> /dev/null; then
        uv tool install zensical
    else
        pip3 install zensical
    fi
fi

# 进入项目根目录
cd "$(dirname "$0")/.."

case "${1:-serve}" in
    "serve")
        print_info "启动 Zensical 开发服务器..."
        zensical serve
        ;;
    "build")
        print_info "构建 Zensical 文档..."
        zensical build --clean
        print_success "文档构建完成，输出在 site/ 目录"
        ;;
    "clean")
        print_info "清理构建文件..."
        rm -rf site/
        print_success "清理完成"
        ;;
    *)
        echo "用法: $0 [serve|build|clean]"
        echo ""
        echo "命令:"
        echo "  serve   启动开发服务器 (默认)"
        echo "  build   构建静态文档"
        echo "  clean   清理构建文件"
        echo ""
        echo "部署由 .github/workflows/docs.yml 在推送到 main 时自动完成"
        exit 1
        ;;
esac