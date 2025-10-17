#!/bin/bash

# 文档构建脚本
# 用于构建和部署 MkDocs 文档

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

# 检查是否安装了 Python
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 未安装"
    exit 1
fi

# 检查是否安装了 pip
if ! command -v pip3 &> /dev/null; then
    print_error "pip3 未安装"
    exit 1
fi

# 进入项目根目录
cd "$(dirname "$0")/.."

print_info "安装 MkDocs 依赖..."
pip3 install -r docs/requirements.txt

case "${1:-serve}" in
    "serve")
        print_info "启动 MkDocs 开发服务器..."
        mkdocs serve
        ;;
    "build")
        print_info "构建 MkDocs 文档..."
        mkdocs build
        print_success "文档构建完成，输出在 site/ 目录"
        ;;
    "deploy")
        print_info "部署文档到 GitHub Pages..."
        mkdocs gh-deploy
        print_success "文档已部署到 GitHub Pages"
        ;;
    "clean")
        print_info "清理构建文件..."
        rm -rf site/
        print_success "清理完成"
        ;;
    *)
        echo "用法: $0 [serve|build|deploy|clean]"
        echo ""
        echo "命令:"
        echo "  serve   启动开发服务器 (默认)"
        echo "  build   构建静态文档"
        echo "  deploy  部署到 GitHub Pages"
        echo "  clean   清理构建文件"
        exit 1
        ;;
esac