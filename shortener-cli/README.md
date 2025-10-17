# Shortener CLI

用于管理短链接的命令行工具。

## 安装

```bash
cargo install --path .
```

或直接运行：

```bash
cargo run -p shortener-cli -- [命令]
```

## 配置

初始化配置：

```bash
shortener-cli init
```

这将在 `~/.config/shortener/config.toml` 创建配置文件。

你也可以使用环境变量：
- `SHORTENER_URL` - API 服务器 URL
- `SHORTENER_KEY` - API 密钥

## 命令

### 配置
- `init` - 初始化配置文件
- `env` - 显示环境变量
- `version` - 显示版本信息

### URL 管理
- `create <url>` - 创建短链接
  - `--code <code>` - 指定自定义短代码
  - `--desc <description>` - 添加描述
- `get <code>` - 获取短链接详情
- `list` - 列出短链接
  - `--all` - 获取所有 URL（自动分页）
  - `--page <n>` - 页码
  - `--psize <n>` - 每页大小
  - `--sort <field>` - 排序字段
  - `--order <asc|desc>` - 排序顺序
- `update <code>` - 更新短链接
  - `--ourl <url>` - 更新原始 URL
  - `--desc <description>` - 更新描述
- `delete <code>` - 删除短链接

## 示例

```bash
# 创建短链接
shortener-cli create https://example.com

# 使用自定义代码创建
shortener-cli create https://example.com --code mylink

# 列出所有 URL
shortener-cli list --all

# 获取详情
shortener-cli get mylink

# 更新 URL
shortener-cli update mylink --ourl https://newurl.com

# 删除 URL
shortener-cli delete mylink
```
