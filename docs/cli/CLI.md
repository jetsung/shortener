# Shortener CLI

命令行工具，用于管理短网址服务。

## 安装

### 从 Git 仓库安装

```bash
cargo install --git https://github.com/jetsung/shortener.git shortener-cli
```

### 从源码安装

```bash
git clone https://github.com/jetsung/shortener.git
cd shortener
cargo install --path shortener-cli
```

## 配置

### 初始化配置

```bash
shortener-cli init
```

这将创建配置文件 `~/.config/shortener/config.toml`：

```toml
[server]
url = "http://localhost:8080"
api_key = "your-api-key"
```

### 使用环境变量

```bash
export SHORTENER_URL="http://localhost:8080"
export SHORTENER_KEY="your-api-key"
```

### 命令行参数

```bash
shortener-cli --url http://localhost:8080 --key your-api-key <command>
```

## 命令

### 基本命令

#### 显示帮助

```bash
shortener-cli --help
shortener-cli <command> --help
```

#### 显示版本

```bash
shortener-cli --version
```

#### 显示环境信息

```bash
shortener-cli env
```

### 短链接管理

#### 创建短链接

```bash
# 创建短链接（自动生成代码）
shortener-cli create https://example.com

# 使用自定义代码
shortener-cli create https://example.com --code mylink

# 添加描述
shortener-cli create https://example.com --code mylink --desc "我的链接"
```

#### 获取短链接详情

```bash
# 通过短码获取详情
shortener-cli get mylink
```

#### 查找短链接

```bash
# 通过原始 URL 查找短链接
shortener-cli find --original_url https://example.com

# 查找所有匹配的短链接（自动翻页）
shortener-cli find --original_url https://example.com --all

# 简写形式
shortener-cli find -r https://example.com
shortener-cli find -r https://example.com -a
```

#### 列出短链接

```bash
# 列出第一页（默认 10 条）
shortener-cli list

# 列出所有短链接（自动翻页）
shortener-cli list --all

# 分页列出
shortener-cli list --page 1 --psize 20

# 排序
shortener-cli list --sort created_at --order desc
shortener-cli list --sort updated_at --order asc

# 按状态过滤
shortener-cli list --status 0  # 仅启用的
shortener-cli list --status 1  # 仅禁用的

# 按原始 URL 过滤
shortener-cli list --original_url https://example.com

# 组合过滤
shortener-cli list --original_url https://example.com --status 0 --page 1 --psize 5

# 指定显示格式
shortener-cli list -f table    # 完整表格（包含所有列）
shortener-cli list -f compact  # 紧凑表格（适合窄终端）
shortener-cli list -f list     # 列表格式（适合很窄的终端）
```

#### 更新短链接

```bash
# 更新原始 URL
shortener-cli update mylink --ourl https://newurl.com

# 更新描述
shortener-cli update mylink --desc "新的描述"

# 更新状态
shortener-cli update mylink --status 1  # 禁用
shortener-cli update mylink --status 0  # 启用

# 同时更新多个字段
shortener-cli update mylink --ourl https://newurl.com --desc "已更新" --status 0
```

#### 删除短链接

```bash
shortener-cli delete mylink
```

## 参数说明

### 全局参数

- `-u, --url <URL>`: 服务器 URL（也可通过 `SHORTENER_URL` 环境变量设置）
- `-k, --key <KEY>`: API 密钥（也可通过 `SHORTENER_KEY` 环境变量设置）

### create 命令

- `<original_url>`: 要缩短的原始 URL（必需）
- `-c, --code <CODE>`: 自定义短码（可选）
- `-d, --desc <DESC>`: 描述（可选）

### get 命令

- `<code>`: 要获取的短码（必需）

### find 命令

- `-r, --original_url <URL>`: 要搜索的原始 URL（必需）
- `-a, --all`: 显示所有匹配结果（自动翻页）

### list 命令

- `-a, --all`: 获取所有短链接（自动翻页）
- `-p, --page <NUM>`: 页码（默认：1）
- `-z, --psize <NUM>`: 每页大小（默认：10）
- `-s, --sort <FIELD>`: 排序字段（如：created_at, updated_at）
- `-o, --order <ORDER>`: 排序方向（asc 或 desc）
- `-t, --status <STATUS>`: 按状态过滤（0=启用, 1=禁用）
- `-r, --original_url <URL>`: 按原始 URL 过滤
- `-f, --format <FORMAT>`: 输出格式（table, compact, list）

### update 命令

- `<code>`: 要更新的短码（必需）
- `-o, --ourl <URL>`: 新的原始 URL
- `-d, --desc <DESC>`: 新的描述
- `-s, --status <STATUS>`: 新的状态（0=启用, 1=禁用）

### delete 命令

- `<code>`: 要删除的短码（必需）

## 使用示例

### 基本工作流

```bash
# 1. 初始化配置
shortener-cli init

# 2. 创建短链接
shortener-cli create https://github.com/jetsung/shortener --code github --desc "项目仓库"

# 3. 查看详情
shortener-cli get github

# 4. 列出所有链接
shortener-cli list --all

# 5. 更新链接
shortener-cli update github --desc "Shortener 项目仓库"

# 6. 查找特定 URL 的所有短链接
shortener-cli find -r https://github.com/jetsung/shortener -a
```

### 高级用法

```bash
# 批量查看最近创建的链接
shortener-cli list --sort created_at --order desc --psize 20

# 查找所有指向某个域名的链接
shortener-cli list --original_url https://example.com --all

# 查看所有禁用的链接
shortener-cli list --status 1 --all

# 分页浏览大量数据
shortener-cli list --page 1 --psize 50
shortener-cli list --page 2 --psize 50
```

### 脚本化使用

```bash
#!/bin/bash

# 设置环境变量
export SHORTENER_URL="https://your-server.com"
export SHORTENER_KEY="your-api-key"

# 批量创建短链接
urls=(
    "https://github.com/jetsung/shortener"
    "https://docs.rs/shortener"
    "https://crates.io/crates/shortener"
)

for url in "${urls[@]}"; do
    echo "Creating short URL for: $url"
    shortener-cli create "$url"
done

# 列出所有创建的链接
echo "All short URLs:"
shortener-cli list --all
```

## 输出格式

### 详情显示

```
ID:           1
Code:         github
Short URL:    http://localhost:8080/github
Original URL: https://github.com/jetsung/shortener
Description:  项目仓库
Status:       0 (Enabled)
Created:      2024-01-15 10:30
Updated:      2024-01-15 10:30
```

### 表格显示格式

#### 完整表格 (-f table)

显示所有信息，Original URL 使用智能截断（域名.../文件名）：

```
┌────┬────────┬─────────────────────────────────────┬─────────────────────────────────────┬─────────────┬─────────┬─────────────────┐
│ ID │ Code   │ Short URL                           │ Original URL                        │ Description │ Status  │ Created         │
├────┼────────┼─────────────────────────────────────┼─────────────────────────────────────┼─────────────┼─────────┼─────────────────┤
│ 1  │ github │ http://localhost:8080/github        │ https://github.com/.../project.git  │ 项目仓库    │ Enabled │ 2024-01-15 10:30│
│ 2  │ script │ http://localhost:8080/script        │ https://example.com/.../install.sh  │ 安装脚本    │ Enabled │ 2024-01-15 10:31│
└────┴────────┴─────────────────────────────────────┴─────────────────────────────────────┴─────────────┴─────────┴─────────────────┘
```

#### 紧凑表格 (-f compact)

显示完整的 Original URL，不显示 Description：

```
┌────────┬─────────────────────────────────────┬──────────────────────────────────────┬─────────┐
│ Code   │ Short URL                           │ Original URL                         │ Status  │
├────────┼─────────────────────────────────────┼──────────────────────────────────────┼─────────┤
│ github │ http://localhost:8080/github        │ https://github.com/jetsung/shortener │ Enabled │
│ docs   │ http://localhost:8080/docs          │ https://docs.rs/shortener            │ Enabled │
└────────┴─────────────────────────────────────┴──────────────────────────────────────┴─────────┘
```

#### 列表格式 (-f list)

适合窄终端（< 100 列），每个条目占多行：

```
1. github (Enabled)
   Short URL: http://localhost:8080/github
   Original:  https://github.com/jetsung/shortener
   Desc:      项目仓库
   Created:   2024-01-15 10:30

2. docs (Enabled)
   Short URL: http://localhost:8080/docs
   Original:  https://docs.rs/shortener
   Desc:      文档
   Created:   2024-01-15 10:31
```

### 默认格式

如果不指定 `--format` 参数，CLI 默认使用列表格式，这是最易读和信息最完整的格式。

## 错误处理

CLI 工具会返回适当的退出码：

- `0`: 成功
- `1`: 错误（网络错误、API 错误、配置错误等）

常见错误：

```bash
# 配置未找到
Error: Configuration not found. Run 'shortener-cli init' first.

# API 密钥无效
Error: Unauthorized: Invalid API key

# 短码不存在
Error: Not found: Short URL 'nonexistent' not found

# 网络连接失败
Error: HTTP request failed: connection refused
```

## 配置文件

配置文件位置：`~/.config/shortener/config.toml`

```toml
[server]
url = "http://localhost:8080"
api_key = "your-secret-api-key"
```

## 环境变量

- `SHORTENER_URL`: 服务器 URL
- `SHORTENER_KEY`: API 密钥

优先级：命令行参数 > 环境变量 > 配置文件

## 开发

### 构建

```bash
cargo build --release -p shortener-cli
```

### 测试

```bash
cargo test -p shortener-cli
```

### 安装开发版本

```bash
cargo install --path shortener-cli --force
```

## 故障排除

### 常见问题

1. **配置文件未找到**
   ```bash
   shortener-cli init
   ```

2. **连接被拒绝**
   - 检查服务器是否运行
   - 检查 URL 是否正确

3. **API 密钥无效**
   - 检查 API 密钥是否正确
   - 重新初始化配置

4. **权限错误**
   - 检查 API 密钥权限
   - 联系管理员

### 调试

启用详细日志：

```bash
RUST_LOG=debug shortener-cli <command>
```

## 许可证

本项目采用 Apache-2.0 许可证。详见 [LICENSE](../LICENSE) 文件。