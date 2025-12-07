#!/bin/bash

# Shortener 一键安装脚本
# 使用方法: curl -sSL https://raw.githubusercontent.com/jetsung/shortener/main/scripts/install.sh | bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
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

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 检查 Rust 和 Cargo
check_rust() {
    if ! command_exists cargo; then
        print_error "Cargo 未找到。请先安装 Rust:"
        echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    # 检查 Rust 版本
    local rust_version=$(rustc --version | cut -d' ' -f2)
    local required_version="1.90.0"

    if ! printf '%s\n%s\n' "$required_version" "$rust_version" | sort -V -C; then
        print_warning "Rust 版本 $rust_version 可能过低，推荐版本 >= $required_version"
        print_info "更新 Rust: rustup update stable"
    fi
}

# 安装函数
install_shortener() {
    local component=$1
    local repo_url="https://github.com/jetsung/shortener.git"

    print_info "正在安装 $component..."

    if cargo install --git "$repo_url" "$component"; then
        print_success "$component 安装成功!"
    else
        print_error "$component 安装失败!"
        return 1
    fi
}

# 验证安装
verify_installation() {
    local component=$1

    if command_exists "$component"; then
        local version=$($component --version 2>/dev/null || echo "unknown")
        print_success "$component 已安装: $version"
        return 0
    else
        print_error "$component 验证失败"
        return 1
    fi
}

# 主函数
main() {
    echo "========================================"
    echo "    Shortener 一键安装脚本"
    echo "========================================"
    echo

    # 检查系统要求
    print_info "检查系统要求..."
    check_rust

    # 询问用户要安装什么
    echo
    print_info "请选择要安装的组件:"
    echo "1) 仅服务器 (shortener-server)"
    echo "2) 仅 CLI 工具 (shortener-cli)"
    echo "3) 全部安装 (推荐)"
    echo "4) 退出"
    echo

    read -p "请输入选择 [1-4]: " choice

    case $choice in
        1)
            install_shortener "shortener-server"
            verify_installation "shortener-server"
            ;;
        2)
            install_shortener "shortener-cli"
            verify_installation "shortener-cli"
            ;;
        3)
            install_shortener "shortener-server"
            install_shortener "shortener-cli"
            verify_installation "shortener-server"
            verify_installation "shortener-cli"
            ;;
        4)
            print_info "安装已取消"
            exit 0
            ;;
        *)
            print_error "无效选择"
            exit 1
            ;;
    esac

    echo
    print_success "安装完成!"
    echo
    print_info "下一步:"

    if command_exists shortener-server; then
        echo "  启动服务器: shortener-server"
        echo "  查看帮助: shortener-server --help"
    fi

    if command_exists shortener-cli; then
        echo "  初始化 CLI: shortener-cli init"
        echo "  查看帮助: shortener-cli --help"
    fi

    echo
    print_info "文档: https://github.com/jetsung/shortener"
    print_info "问题反馈: https://github.com/jetsung/shortener/issues"
}

# 运行主函数
main "$@"