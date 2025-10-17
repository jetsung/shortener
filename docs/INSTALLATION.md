# 安装指南

本文档提供了 Shortener 项目的详细安装说明，包括多种安装方式和平台支持。

## 目录

- [系统要求](#系统要求)
- [Cargo 安装方式](#cargo-安装方式)
- [Docker 安装](#docker-安装)
- [预构建二进制文件](#预构建二进制文件)
- [DEB 包安装](#deb-包安装)
- [从源码构建](#从源码构建)
- [验证安装](#验证安装)
- [故障排除](#故障排除)

## 系统要求

### 最低要求

- **操作系统**: Linux, macOS, Windows
- **Rust**: 1.90 或更高版本
- **内存**: 最少 512MB RAM
- **存储**: 最少 100MB 可用空间

### 推荐配置

- **操作系统**: Linux (Ubuntu 20.04+, CentOS 8+, Debian 11+)
- **Rust**: 最新稳定版
- **内存**: 1GB+ RAM
- **存储**: 1GB+ 可用空间
- **数据库**: PostgreSQL 或 MySQL（生产环境）

## Cargo 安装方式

### 从 Git 仓库安装（推荐）

这是最简单和最新的安装方式，直接从 GitHub 仓库安装最新版本。

#### 安装最新版本

```bash
# 安装服务器
cargo install --git https://github.com/jetsung/shortener.git shortener-server

# 安装 CLI 工具
cargo install --git https://github.com/jetsung/shortener.git shortener-cli
```

#### 安装指定版本

```bash
# 安装特定标签版本
cargo install --git https://github.com/jetsung/shortener.git --tag v1.0.0 shortener-server
cargo install --git https://github.com/jetsung/shortener.git --tag v1.0.0 shortener-cli

# 查看可用标签
git ls-remote --tags https://github.com/jetsung/shortener.git
```

#### 安装指定分支

```bash
# 安装主分支（开发版本）
cargo install --git https://github.com/jetsung/shortener.git --branch main shortener-server

# 安装特定分支
cargo install --git https://github.com/jetsung/shortener.git --branch develop shortener-cli
```

#### 强制重新安装

```bash
# 强制重新安装最新版本
cargo install --git https://github.com/jetsung/shortener.git --force shortener-server
cargo install --git https://github.com/jetsung/shortener.git --force shortener-cli
```

#### 安装到自定义目录

```bash
# 安装到指定目录
cargo install --git https://github.com/jetsung/shortener.git --root /opt/shortener shortener-server

# 设置 PATH
export PATH="/opt/shortener/bin:$PATH"
```

### 从本地源码安装

如果你已经克隆了仓库或想要修改源码：

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 安装服务器
cargo install --path shortener-server

# 安装 CLI 工具
cargo install --path shortener-cli

# 安装到自定义目录
cargo install --path shortener-server --root /usr/local
```

### 从 crates.io 安装

当项目发布到 crates.io 后，可以使用以下方式安装：

```bash
# 安装服务器
cargo install shortener-server

# 安装 CLI 工具
cargo install shortener-cli

# 安装指定版本
cargo install shortener-server --version 1.0.0
```

## Docker 安装

### 使用 Docker Compose（推荐）

```bash
# 下载 docker-compose.yml
curl -O https://raw.githubusercontent.com/jetsung/shortener/main/docker/docker-compose.yml

# 启动服务
docker compose up -d

# 查看日志
docker compose logs -f shortener-server
```

### 使用 Docker 命令

```bash
# 拉取镜像
docker pull jetsung/shortener-server:latest

# 运行容器
docker run -d \
  --name shortener-server \
  -p 8080:8080 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/config:/app/config \
  jetsung/shortener-server:latest
```

详见 [Docker 部署指南](DOCKER.md)。

## 预构建二进制文件

从 [GitHub Releases](https://github.com/jetsung/shortener/releases) 下载预构建的二进制文件。

### Linux

```bash
# x86_64
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-x86_64-unknown-linux-musl.tar.gz
tar xzf shortener-server-v1.0.0-x86_64-unknown-linux-musl.tar.gz
sudo mv shortener-server /usr/local/bin/

# ARM64
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-aarch64-unknown-linux-musl.tar.gz
tar xzf shortener-server-v1.0.0-aarch64-unknown-linux-musl.tar.gz
sudo mv shortener-server /usr/local/bin/
```

### macOS

```bash
# Intel Mac
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-x86_64-apple-darwin.tar.gz
tar xzf shortener-server-v1.0.0-x86_64-apple-darwin.tar.gz
sudo mv shortener-server /usr/local/bin/

# Apple Silicon
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-aarch64-apple-darwin.tar.gz
tar xzf shortener-server-v1.0.0-aarch64-apple-darwin.tar.gz
sudo mv shortener-server /usr/local/bin/
```

### Windows

```powershell
# 下载并解压
Invoke-WebRequest -Uri "https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server-v1.0.0-x86_64-pc-windows-msvc.zip" -OutFile "shortener-server.zip"
Expand-Archive -Path "shortener-server.zip" -DestinationPath "C:\Program Files\Shortener"

# 添加到 PATH
$env:PATH += ";C:\Program Files\Shortener"
```

## DEB 包安装

适用于 Debian、Ubuntu 及其衍生发行版。

### 下载并安装

```bash
# 下载 DEB 包
wget https://github.com/jetsung/shortener/releases/download/v1.0.0/shortener-server_1.0.0_amd64.deb

# 安装
sudo apt install ./shortener-server_1.0.0_amd64.deb

# 或使用 dpkg
sudo dpkg -i shortener-server_1.0.0_amd64.deb
sudo apt-get install -f  # 修复依赖关系
```

### 服务管理

```bash
# 启动服务
sudo systemctl start shortener-server

# 开机自启
sudo systemctl enable shortener-server

# 查看状态
sudo systemctl status shortener-server

# 查看日志
sudo journalctl -u shortener-server -f
```

### 配置文件位置

- 配置文件: `/opt/shortener-server/config/config.toml`
- 数据目录: `/opt/shortener-server/data/`
- 日志文件: `/var/log/shortener-server/`
- 服务文件: `/etc/systemd/system/shortener-server.service`

详见 [DEB 打包指南](DEB_PACKAGING_SIMPLIFIED.md)。

## 从源码构建

### 克隆仓库

```bash
git clone https://github.com/jetsung/shortener.git
cd shortener
```

### 构建发布版本

```bash
# 构建所有包
cargo build --release

# 仅构建服务器
cargo build --release -p shortener-server

# 仅构建 CLI
cargo build --release -p shortener-cli
```

### 安装到系统

```bash
# 复制二进制文件
sudo cp target/release/shortener-server /usr/local/bin/
sudo cp target/release/shortener-cli /usr/local/bin/

# 设置权限
sudo chmod +x /usr/local/bin/shortener-server
sudo chmod +x /usr/local/bin/shortener-cli
```

### 交叉编译

```bash
# 安装交叉编译工具链
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# 交叉编译
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
```

## 验证安装

### 检查版本

```bash
# 检查服务器版本
shortener-server --version

# 检查 CLI 版本
shortener-cli --version
```

### 测试运行

```bash
# 测试服务器（使用默认配置）
shortener-server --help

# 测试 CLI
shortener-cli --help

# 测试 CLI 新功能
shortener-cli find --help
```

### 完整测试

```bash
# 1. 启动服务器（后台运行）
shortener-server &
SERVER_PID=$!

# 2. 等待服务器启动
sleep 2

# 3. 初始化 CLI
shortener-cli init

# 4. 创建测试链接
shortener-cli create https://example.com

# 5. 测试查找功能
shortener-cli find --original_url https://example.com

# 6. 停止服务器
kill $SERVER_PID
```

## 故障排除

### 常见问题

#### Rust 版本过低

```bash
# 错误信息
error: package `shortener-server v1.0.0` cannot be built because it requires rustc 1.90 or newer

# 解决方案
rustup update stable
```

#### 缺少系统依赖

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel

# macOS
xcode-select --install
```

#### 网络连接问题

```bash
# 使用国内镜像源
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup

# 配置 Cargo 镜像
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << EOF
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
EOF
```

#### 权限问题

```bash
# 如果安装到系统目录需要 sudo
sudo cargo install --git https://github.com/jetsung/shortener.git --root /usr/local shortener-server

# 或安装到用户目录
cargo install --git https://github.com/jetsung/shortener.git shortener-server
# 默认安装到 ~/.cargo/bin/
```

#### 磁盘空间不足

```bash
# 清理 Cargo 缓存
cargo clean

# 清理注册表缓存
rm -rf ~/.cargo/registry/cache/
rm -rf ~/.cargo/git/db/

# 检查磁盘空间
df -h
```

### 获取帮助

如果遇到其他问题：

1. 查看 [项目文档](https://jetsung.github.io/shortener)
2. 搜索 [GitHub Issues](https://github.com/jetsung/shortener/issues)
3. 创建新的 [Issue](https://github.com/jetsung/shortener/issues/new)
4. 查看 [讨论区](https://github.com/jetsung/shortener/discussions)

### 日志调试

```bash
# 启用详细日志
RUST_LOG=debug shortener-server

# 启用特定模块日志
RUST_LOG=shortener_server::handlers=debug shortener-server

# 保存日志到文件
shortener-server 2>&1 | tee shortener.log
```

## 卸载

### Cargo 安装的程序

```bash
# 卸载服务器
cargo uninstall shortener-server

# 卸载 CLI
cargo uninstall shortener-cli

# 查看已安装的程序
cargo install --list
```

### 手动安装的程序

```bash
# 删除二进制文件
sudo rm /usr/local/bin/shortener-server
sudo rm /usr/local/bin/shortener-cli

# 删除配置文件（可选）
rm -rf ~/.config/shortener/
```

### DEB 包

```bash
# 卸载包
sudo apt remove shortener-server

# 完全删除（包括配置文件）
sudo apt purge shortener-server
```

---

**需要帮助？** 查看 [项目文档](https://jetsung.github.io/shortener) 或在 [GitHub](https://github.com/jetsung/shortener/issues) 上提问。