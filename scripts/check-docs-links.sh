#!/bin/bash

# 文档链接检查脚本
# 递归检查 docs 目录中所有 .md 文件的内部相对链接是否有效

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

# 进入项目根目录
cd "$(dirname "$0")/.."

print_info "检查文档链接..."

error_count=0
checked_count=0

# 递归检查 docs 下所有 .md 文件
while IFS= read -r doc_file; do
    # 获取文件所在目录（相对于项目根）
    doc_dir=$(dirname "$doc_file")

    # 提取所有指向 .md 的相对链接
    while IFS= read -r raw; do
        [ -z "$raw" ] && continue

        # raw 形如 "](deployment/DOCKER.md)"，剥离前缀 ]( 与尾部 )
        target=$(sed 's/^](//; s/)$//' <<< "$raw")

        # 去掉锚点(#...)和查询参数(?...)
        target="${target%%#*}"
        target="${target%%\?*}"

        # 跳过外部链接
        case "$target" in
            http://*|https://*|mailto:*|ftp://*) continue ;;
        esac
        # 跳过纯锚点
        [ -z "$target" ] && continue

        checked_count=$((checked_count + 1))

        # 相对于当前文件所在目录解析
        if [ ! -f "$doc_dir/$target" ]; then
            # 尝试去掉 ./ 前缀
            t2="${target#./}"
            if [ ! -f "$doc_dir/$t2" ]; then
                print_error "死链接: $doc_file -> $target"
                error_count=$((error_count + 1))
            fi
        fi
    done < <(grep -oE "\]\([^)]*\.md([#?][^)]*)?\)" "$doc_file" 2>/dev/null || true)
done < <(find docs -name "*.md" -type f)

echo ""
print_info "共检查 $checked_count 个内部链接"
if [[ $error_count -eq 0 ]]; then
    print_success "所有文档链接检查通过！"
else
    print_error "发现 $error_count 个死链接"
    exit 1
fi