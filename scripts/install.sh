#!/usr/bin/env bash

#============================================================
# File: install.sh
# Description: Shortener 一键安装脚本（预编译发行版，无需 Rust）
# Usage:
#   交互式: curl -sSL https://shortener.gcli.cn/install.sh | bash
#   非交互: curl -sSL https://shortener.gcli.cn/install.sh | bash -s -- cli
#           组件参数: server | cli | all
#============================================================

if [[ -n "${DEBUG:-}" ]]; then
    set -eux
else
    set -euo pipefail
fi

CDN_URL="${CDN:-https://filetas.asfd.cn}"
REPO="jetsung/shortener"
# release tag 固定前缀，如 shortener-server-v0.2.1，cli 资产与其共用版本号
TAG_PREFIX="shortener-server-"
INSTALL_DIR="/usr/local/bin"
# 中国网络时由 main 置位，启用 CDN 下载
USE_CDN=""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

check_is_command() {
    command -v "$1" >/dev/null 2>&1
}

USER_ID="$(id -u)"

sudo_exec() {
    if [[ "$USER_ID" -ne 0 ]]; then
        sudo "$@"
    else
        "$@"
    fi
}

check_in_china() {
    if [[ -n "${CN:-}" ]]; then
        return 0 # 手动指定
    fi
    if [[ "$(curl -s -m 3 -o /dev/null -w "%{http_code}" https://www.google.com)" == "000" ]]; then
        return 0 # 中国网络
    fi
    return 1 # 非中国网络
}

# 组合下载地址：启用 CDN 时以 "/" 连接前缀与原始完整 URL，
# 格式为 https://<cdn>/https://github.com/...；NO_HTTPS=1 时去掉内层 https://
cdn_join() {
    local url="$1"
    if [[ -n "${USE_CDN:-}" ]]; then
        if [[ -n "${NO_HTTPS:-}" ]]; then
            echo "${CDN_URL%/}/${url#https://}"
        else
            echo "${CDN_URL%/}/${url}"
        fi
    else
        echo "$url"
    fi
}

# 获取最新 release 的版本号（tag 去掉前缀），如 v0.2.1
get_latest_version() {
    local api_url
    api_url=$(cdn_join "https://api.github.com/repos/${REPO}/releases/latest")

    local tag
    tag=$(curl -fsSL "$api_url" | jq -r '.tag_name // empty')
    if [[ -z "$tag" ]]; then
        print_error "无法获取 ${REPO} 的最新 release 信息"
        exit 1
    fi

    echo "${tag#"${TAG_PREFIX}"}"
}

# 目标三元组：linux 一律使用 musl（静态链接，glibc 系统同样可运行，
# 且仓库发行版仅提供 musl 构建）；mac/windows 使用官方构建目标
detect_target() {
    local os arch
    os="$(uname | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$arch" in
        x86_64|i686)   arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        armv7l)        arch="armv7" ;;
        loongarch64)   arch="loongarch64" ;;
        *) print_error "不支持的架构: $arch"; exit 1 ;;
    esac

    case "$os" in
        linux)
            echo "${arch}-unknown-linux-musl .tar.xz"
            ;;
        darwin)
            echo "${arch}-apple-darwin .tar.xz"
            ;;
        *)
            print_error "暂不支持的系统: $os (仅支持 Linux / macOS)"
            exit 1
            ;;
    esac
}

# 下载并安装单个组件
install_component() {
    local component="$1" version="$2"
    local target ext download_url download_file tmp_dir
    read -r target ext <<<"$(detect_target)"

    download_file="shortener-${component}-${version}-${target}${ext}"
    download_url="https://github.com/${REPO}/releases/download/${TAG_PREFIX}${version}/${download_file}"
    download_url=$(cdn_join "$download_url")

    print_info "正在安装 shortener-${component} (${version}) ..."
    echo "下载地址: $download_url"

    tmp_dir=$(mktemp -d /tmp/shortener.XXXXXX)
    # shellcheck disable=SC2329
    cleanup() { rm -rf -- "$tmp_dir"; }
    trap cleanup RETURN

    if ! curl -fL --retry 3 -o "$tmp_dir/$download_file" "$download_url"; then
        print_error "下载失败: $download_url"
        return 1
    fi

    if ! tar -xJf "$tmp_dir/$download_file" -C "$tmp_dir"; then
        print_error "解压失败: $download_file"
        return 1
    fi

    if [[ ! -f "$tmp_dir/shortener-${component}" ]]; then
        print_error "安装包中未找到 shortener-${component}，包内文件如下："
        find "$tmp_dir" -type f | head -n 20
        return 1
    fi

    sudo_exec install -m 0755 "$tmp_dir/shortener-${component}" "${INSTALL_DIR}/shortener-${component}"
    print_success "shortener-${component} 安装成功!"
}

# 验证安装
verify_installation() {
    local component="$1"

    if check_is_command "shortener-${component}"; then
        local version
        version=$(shortener-${component} --version 2>/dev/null || echo "unknown")
        print_success "shortener-${component} 已安装: $version"
    else
        print_warning "shortener-${component} 未在 PATH 中找到（可能需要重新登录终端）"
    fi
}

main() {
    echo "========================================"
    echo "    Shortener 一键安装脚本"
    echo "========================================"
    echo

    for cmd in curl jq tar xz; do
        if ! check_is_command "$cmd"; then
            print_error "缺少依赖命令: $cmd，请先安装"
            exit 1
        fi
    done

    if check_in_china; then
        USE_CDN=1
        print_info "已检测到中国网络，将使用 CDN 加速下载"
    fi

    # 确定要安装的组件: 优先使用命令行参数, 否则交互式选择
    local components=()
    if [[ $# -gt 0 ]]; then
        case "$1" in
            server) components=(server) ;;
            cli)    components=(cli) ;;
            all)    components=(server cli) ;;
            *)
                print_error "未知组件: $1 (可选: server | cli | all)"
                exit 1
                ;;
        esac
    else
        echo
        print_info "请选择要安装的组件:"
        echo "1) 仅服务器 (shortener-server)"
        echo "2) 仅 CLI 工具 (shortener-cli)"
        echo "3) 全部安装 (推荐)"
        echo "4) 退出"
        echo

        local choice
        read -r -p "请输入选择 [1-4]: " choice
        case "$choice" in
            1) components=(server) ;;
            2) components=(cli) ;;
            3) components=(server cli) ;;
            4) print_info "安装已取消"; exit 0 ;;
            *) print_error "无效选择"; exit 1 ;;
        esac
    fi

    local version
    version=$(get_latest_version)
    print_info "最新版本: ${version}"

    local failed=0
    for component in "${components[@]}"; do
        install_component "$component" "$version" || failed=1
    done

    echo
    for component in "${components[@]}"; do
        verify_installation "$component"
    done

    if [[ "$failed" -ne 0 ]]; then
        exit 1
    fi

    echo
    print_success "安装完成!"
    echo
    print_info "下一步:"

    if check_is_command "shortener-server"; then
        echo "  启动服务器: shortener-server"
        echo "  查看帮助: shortener-server --help"
    fi

    if check_is_command "shortener-cli"; then
        echo "  初始化 CLI: shortener-cli init"
        echo "  查看帮助: shortener-cli --help"
    fi

    echo
    print_info "文档: https://shortener.gcli.cn"
    print_info "问题反馈: https://github.com/jetsung/shortener/issues"
}

main "$@"
