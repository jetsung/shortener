# Shortener CLI

命令行工具，用于管理短链接服务。

## 📚 完整文档

请查看 [CLI 文档](../docs/cli/CLI.md) 获取完整的使用说明。

## 🚀 快速开始

```bash
# 构建
cargo build --release -p shortener-cli

# 安装
cargo install --path .

# 初始化
shortener-cli init

# 创建短链接
shortener-cli create https://example.com

# 获取详情
shortener-cli get abc123

# 列出所有
shortener-cli list --all
```

## 📖 更多信息

- [完整文档](../docs/cli/CLI.md)
- [命令参考](../docs/cli/USAGE.md)
- [在线文档](https://jetsung.github.io/shortener/cli/CLI/)
