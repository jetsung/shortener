# Shortener CLI 使用指南

shortener-cli 是一个命令行工具，用于管理短网址服务。

## 安装

```bash
cd shortener-cli
cargo build --release
sudo cp target/release/shortener-cli /usr/local/bin/
```

## 配置

### 初始化配置

```bash
shortener-cli init -u http://localhost:8080 -k your-api-key
```

配置文件将保存在 `~/.config/shortener-cli/config.toml`

### 环境变量

也可以使用环境变量：

```bash
export SHORTENER_URL=http://localhost:8080
export SHORTENER_KEY=your-api-key
```

### 命令行参数

或者在每次命令中指定：

```bash
shortener-cli --url http://localhost:8080 --key your-api-key <command>
```

## 基本命令

### 查看帮助

```bash
shortener-cli --help
shortener-cli <command> --help
```

### 查看版本

```bash
shortener-cli version
```

### 查看环境信息

```bash
shortener-cli env
```

## 短网址管理

### 创建短网址

```bash
# 自动生成短码
shortener-cli create https://example.com

# 指定短码
shortener-cli create https://example.com -c mycode

# 添加描述
shortener-cli create https://example.com -c mycode -d "我的链接"
```

### 查看短网址

```bash
shortener-cli get mycode
```

### 列表查询

```bash
# 查看第一页（默认 10 条）
shortener-cli list

# 指定页码和每页数量
shortener-cli list -p 2 -z 20

# 查看所有（自动翻页）
shortener-cli list -a

# 按状态筛选
shortener-cli list -t 0  # 0=启用, 1=禁用

# 按原始 URL 筛选
shortener-cli list -r "example.com"

# 排序
shortener-cli list -s created_at -o desc
```

### 输出格式

```bash
# 表格格式（完整）
shortener-cli list -f table

# 紧凑格式
shortener-cli list -f compact

# 列表格式（默认）
shortener-cli list -f list
```

### 查找短网址

```bash
# 按原始 URL 查找
shortener-cli find -r "https://example.com"

# 查找所有匹配项
shortener-cli find -r "example.com" -a
```

### 更新短网址

```bash
# 更新原始 URL
shortener-cli update mycode -o https://new-url.com

# 更新描述
shortener-cli update mycode -d "新的描述"

# 更新状态
shortener-cli update mycode -s 1  # 禁用

# 同时更新多个字段
shortener-cli update mycode -o https://new-url.com -d "新描述" -s 0
```

### 删除短网址

```bash
shortener-cli delete mycode
```

## 使用示例

### 示例 1: 快速创建短网址

```bash
$ shortener-cli create https://github.com/jetsung -c github -d "我的 GitHub"
✓ Short URL created successfully!

ID:           1
Code:         github
Short URL:    http://localhost:8080/github
Original URL: https://github.com/jetsung
Description:  我的 GitHub
Status:       0 (Enabled)
Created:      2024-01-15T08:30:00Z
Updated:      2024-01-15T08:30:00Z
```

### 示例 2: 批量查看

```bash
$ shortener-cli list -a
Total: 5 short URLs

1. github (Enabled)
   Short URL: http://localhost:8080/github
   Original:  https://github.com/jetsung
   Desc:      我的 GitHub
   Created:   2024-01-15 08:30

2. blog (Enabled)
   Short URL: http://localhost:8080/blog
   Original:  https://blog.example.com
   Created:   2024-01-15 09:00
...
```

### 示例 3: 查找和更新

```bash
# 查找包含 github 的链接
$ shortener-cli find -r "github"
Found 2 short URL(s) for: github

1. github (Enabled)
   Short URL: http://localhost:8080/github
   Original:  https://github.com/jetsung
   ...

# 更新其中一个
$ shortener-cli update github -d "我的 GitHub 主页"
✓ Short URL updated successfully!
```

## 字段说明

### 状态值
- `0` - 启用 (Enabled)
- `1` - 禁用 (Disabled)

### 排序字段
- `id` - 按 ID 排序
- `short_code` - 按短码排序
- `created_at` - 按创建时间排序（默认）
- `updated_at` - 按更新时间排序

### 排序方向
- `asc` - 升序
- `desc` - 降序（默认）

## 故障排除

### 连接错误

```bash
Error: HTTP request failed: ...
```

检查：
1. 服务器是否运行
2. URL 是否正确
3. 网络连接是否正常

### 认证错误

```bash
Error: Unauthorized: Invalid API key
```

检查：
1. API Key 是否正确
2. 是否已配置或传递 API Key

### 未找到

```bash
Error: Not found: Short URL 'xxx' not found
```

检查短码是否存在：
```bash
shortener-cli list | grep xxx
```

## 相关文档

- [API 变更说明](../migration/API_CHANGES.md)
- [服务器配置](../server/REFACTORING.md)
