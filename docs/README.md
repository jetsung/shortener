# 文档目录

本目录包含 Shortener 项目的完整文档，使用 MkDocs 构建。

## 文档结构

```
docs/
├── index.md                        # 文档首页和导航
├── INSTALLATION.md                 # 安装指南
├── CONFIGURATION.md                # 配置指南
├── API.md                         # API 文档
├── DEPLOYMENT.md                  # 部署指南
├── DOCKER.md                      # Docker 部署
├── DEB_PACKAGING_SIMPLIFIED.md    # DEB 包安装
├── requirements.txt               # MkDocs 依赖
└── README.md                      # 本文件
```

## 文档内容

### 快速开始
- **[安装指南](INSTALLATION.md)** - 详细的安装说明，包括多种安装方式
- **[配置指南](CONFIGURATION.md)** - 服务器和 CLI 配置选项

### 使用指南
- **[API 文档](API.md)** - RESTful API 参考和示例

### 部署指南
- **[部署概述](DEPLOYMENT.md)** - 生产环境部署最佳实践
- **[Docker 部署](DOCKER.md)** - 使用 Docker 和 Docker Compose
- **[DEB 包安装](DEB_PACKAGING_SIMPLIFIED.md)** - Debian/Ubuntu 系统安装

## 构建文档

### 本地开发

```bash
# 安装依赖
pip install -r requirements.txt

# 启动开发服务器
mkdocs serve

# 访问 http://127.0.0.1:8000
```

### 使用构建脚本

```bash
# 启动开发服务器
../scripts/build-docs.sh serve

# 构建静态文档
../scripts/build-docs.sh build

# 部署到 GitHub Pages
../scripts/build-docs.sh deploy

# 清理构建文件
../scripts/build-docs.sh clean
```

## 文档编写指南

### 格式规范

1. **文件命名**：使用大写字母和下划线，如 `INSTALLATION.md`
2. **标题层级**：使用标准的 Markdown 标题层级
3. **代码块**：指定语言类型，如 `bash`、`toml`、`json`
4. **链接**：使用相对路径链接到其他文档

### 内容原则

1. **避免重复**：不同文档间避免重复内容，使用链接引用
2. **保持更新**：代码示例和配置要与实际代码保持同步
3. **用户友好**：提供清晰的步骤和示例
4. **结构清晰**：使用目录和小节组织内容

### 更新流程

1. 修改相应的 Markdown 文件
2. 本地测试：`mkdocs serve`
3. 提交更改到 Git
4. GitHub Actions 会自动部署到 GitHub Pages

## 在线文档

- **GitHub Pages**: https://jetsung.github.io/shortener
- **源码仓库**: https://github.com/jetsung/shortener

## 贡献

欢迎改进文档！请：

1. Fork 仓库
2. 创建分支：`git checkout -b improve-docs`
3. 修改文档
4. 测试构建：`mkdocs serve`
5. 提交 PR

---

**注意**：本目录中的文档会自动同步到在线文档站点。