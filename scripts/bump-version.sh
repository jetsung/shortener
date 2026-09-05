#!/bin/bash

# 版本号同步脚本
# 把同一版本号写入 Cargo.toml / openapi.yml / shortener-frontend/package.json，
# 避免三处各自手工维护产生漂移。
#
# 用法:
#   ./scripts/bump-version.sh          # 显示三处当前版本号，不一致时以非 0 退出
#   ./scripts/bump-version.sh 0.2.0    # 同步三处为 0.2.0

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

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# 进入项目根目录
cd "$(dirname "$0")/.."

# 打印三处当前版本号
print_versions() {
    printf '  %-24s %s\n' "Cargo.toml" "$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
    printf '  %-24s %s\n' "openapi.yml" "$(sed -n 's/^  version: //p' openapi.yml | head -1)"
    printf '  %-24s %s\n' "frontend/package.json" "$(sed -n 's/^  "version": "\(.*\)",/\1/p' shortener-frontend/package.json | head -1)"
}

VERSION="$1"

# 不带参数时只做展示与一致性检查
if [[ -z "$VERSION" ]]; then
    print_info "当前版本号："
    print_versions

    cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
    openapi_version=$(sed -n 's/^  version: //p' openapi.yml | head -1)
    pkg_version=$(sed -n 's/^  "version": "\(.*\)",/\1/p' shortener-frontend/package.json | head -1)

    if [[ "$cargo_version" != "$openapi_version" || "$cargo_version" != "$pkg_version" ]]; then
        print_warning "版本号不一致，运行 $0 <版本号> 同步（以 Cargo.toml 为基准可运行 $0 $cargo_version）"
        exit 1
    fi

    print_success "三处版本号已对齐：$cargo_version"
    exit 0
fi

# 语义化版本（允许 0.2.0 与 0.2.0-beta.1 这类预发布后缀；
# 也允许 Git 标签的 v 前缀，如 v0.2.0-preview.1 / v0.2.0-rc.1 / v0.2.0-alpha.1 / v0.2.0-beta.1）
if [[ ! "$VERSION" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
    print_error "版本号格式非法: $VERSION（应形如 0.2.0、0.2.0-beta.1 或 v0.2.0-rc.1）"
    exit 1
fi

# 剥离可选的 v 前缀：Cargo.toml / openapi.yml / package.json 中的版本号不带 v（v 仅用于 Git 标签）
VERSION="${VERSION#v}"

# 只替换文件中第一次匹配到的行，避免误伤其他同名键
replace_first() {
    local file="$1" pattern="$2" replacement="$3"

    if [[ ! -f "$file" ]]; then
        print_error "文件不存在: $file"
        exit 1
    fi

    awk -v pat="$pattern" -v rep="$replacement" '
        !replaced && $0 ~ pat { print rep; replaced = 1; next }
        { print }
    ' "$file" > "$file.tmp"

    if ! grep -q -- "$replacement" "$file.tmp"; then
        rm -f "$file.tmp"
        print_error "未能在 $file 中匹配 /$pattern/"
        exit 1
    fi

    mv "$file.tmp" "$file"
}

replace_first "Cargo.toml" '^version = "' "version = \"$VERSION\""
replace_first "openapi.yml" '^  version: ' "  version: $VERSION"
replace_first "shortener-frontend/package.json" '^  "version": ' "  \"version\": \"$VERSION\","

print_info "同步后的版本号："
print_versions

print_success "版本号已同步为 $VERSION"
