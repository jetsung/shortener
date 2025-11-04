#!/bin/bash
# 快速查看文档脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Shortener 文档查看工具 ===${NC}"
echo ""

# 检查 mkdocs 是否安装
if ! command -v mkdocs &> /dev/null; then
    echo -e "${YELLOW}MkDocs 未安装，正在安装...${NC}"
    pip install -r docs/requirements.txt
fi

# 显示选项
echo "请选择操作："
echo "1) 启动文档服务器 (http://127.0.0.1:8000)"
echo "2) 构建静态文档"
echo "3) 部署到 GitHub Pages"
echo "4) 清理构建文件"
echo ""
read -p "请输入选项 (1-4): " choice

case $choice in
    1)
        echo -e "${GREEN}启动文档服务器...${NC}"
        echo -e "${BLUE}访问 http://127.0.0.1:8000 查看文档${NC}"
        echo -e "${YELLOW}按 Ctrl+C 停止服务器${NC}"
        mkdocs serve
        ;;
    2)
        echo -e "${GREEN}构建静态文档...${NC}"
        mkdocs build
        echo -e "${GREEN}✓ 文档已构建到 site/ 目录${NC}"
        ;;
    3)
        echo -e "${GREEN}部署到 GitHub Pages...${NC}"
        mkdocs gh-deploy
        echo -e "${GREEN}✓ 文档已部署${NC}"
        ;;
    4)
        echo -e "${GREEN}清理构建文件...${NC}"
        rm -rf site/
        echo -e "${GREEN}✓ 清理完成${NC}"
        ;;
    *)
        echo -e "${YELLOW}无效的选项${NC}"
        exit 1
        ;;
esac
