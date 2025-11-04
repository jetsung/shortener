# CLI 工具文档

Shortener CLI 是一个功能完整的命令行工具，用于管理短链接服务。

## 📚 文档列表

- [CLI 完整文档](CLI.md) - 详细的使用说明和示例
- [命令参考](USAGE.md) - 所有命令的参考文档

## 🚀 快速开始

### 安装

```bash
# 从 Git 仓库安装
cargo install --git https://github.com/jetsung/shortener.git shortener-cli

# 从源码安装
git clone https://github.com/jetsung/shortener.git
cd shortener
cargo install --path shortener-cli
```

### 初始化配置

```bash
shortener-cli init
```

### 基本使用

```bash
# 创建短链接
shortener-cli create https://example.com

# 获取短链接详情
shortener-cli get abc123

# 列出所有短链接
shortener-cli list --all

# 更新短链接
shortener-cli update abc123 --desc "新的描述"

# 删除短链接
shortener-cli delete abc123
```

## 📖 详细文档

查看 [CLI 完整文档](CLI.md) 了解更多功能和使用方法。

## 🔧 配置

CLI 支持多种配置方式：

1. **配置文件**: `~/.config/shortener/config.toml`
2. **环境变量**: `SHORTENER_URL` 和 `SHORTENER_KEY`
3. **命令行参数**: `--url` 和 `--key`

优先级：命令行参数 > 环境变量 > 配置文件

## 💡 提示

- 使用 `--help` 查看任何命令的帮助信息
- 使用 `--all` 参数自动翻页获取所有结果
- 使用 `-f` 参数指定输出格式（table/compact/list）
