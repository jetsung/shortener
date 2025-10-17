#!/bin/bash

# 文档链接检查脚本
# 检查 docs 目录中的内部链接是否有效

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

# 检查的文档文件
docs_files=(
    "docs/index.md"
    "docs/INSTALLATION.md"
    "docs/CONFIGURATION.md"
    "docs/API.md"
    "docs/DEPLOYMENT.md"
    "docs/DOCKER.md"
    "docs/DEB_PACKAGING_SIMPLIFIED.md"
    "docs/README.md"
)

# 存在的文档文件
existing_docs=(
    "API.md"
    "CONFIGURATION.md"
    "DEB_PACKAGING_SIMPLIFIED.md"
    "DEPLOYMENT.md"
    "DOCKER.md"
    "index.md"
    "INSTALLATION.md"
    "README.md"
)

error_count=0

# 检查每个文档文件
for doc_file in "${docs_files[@]}"; do
    if [[ ! -f "$doc_file" ]]; then
        print_warning "文件不存在: $doc_file"
        continue
    fi
    
    print_info "检查文件: $doc_file"
    
    # 提取所有 .md 文件的链接
    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            # 提取文件名
            filename=$(echo "$line" | sed 's/.*](\([^)]*\.md\)).*/\1/')
            
            # 检查是否是相对路径的 .md 文件
            if [[ "$filename" == *.md ]] && [[ "$filename" != http* ]] && [[ "$filename" != https* ]]; then
                # 检查文件是否存在
                if [[ ! " ${existing_docs[@]} " =~ " ${filename} " ]]; then
                    print_error "死链接发现在 $doc_file: $filename"
                    ((error_count++))
                fi
            fi
        fi
    done < <(grep -o '\[.*\]([^)]*\.md)' "$doc_file" 2>/dev/null || true)
done

echo ""
if [[ $error_count -eq 0 ]]; then
    print_success "所有文档链接检查通过！"
else
    print_error "发现 $error_count 个死链接"
    exit 1
fi