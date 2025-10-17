# DEB 包打包指南

本文档介绍如何为 Shortener 项目创建 Debian (.deb) 安装包。

## 目录

- [前提条件](#前提条件)
- [打包脚本说明](#打包脚本说明)
- [使用 cargo-deb 打包](#使用-cargo-deb-打包)
- [手动打包](#手动打包)
- [安装和卸载](#安装和卸载)
- [测试](#测试)

## 前提条件

### 安装 cargo-deb

```bash
cargo install cargo-deb
```

### 系统要求

- Debian/Ubuntu 系统或兼容系统
- dpkg 工具
- 构建工具链（gcc, make 等）

## 打包脚本说明

项目在 `scripts/` 目录下提供了四个用于 deb 包安装/卸载的脚本：

### preinstall.sh

安装前执行的脚本，主要功能：
- 备份现有的配置文件（如果存在）

```bash
#!/usr/bin/env bash
set -e

if [[ -f /opt/shortener-server/config/config.toml ]]; then
  mv /opt/shortener-server/config/config.toml /opt/shortener-server/config/config.toml.bak
fi

exit 0
```

### postinstall.sh

安装后执行的脚本，主要功能：
- 恢复备份的配置文件
- 创建默认配置文件（如果不存在）
- 创建符号链接到 `/usr/local/bin/`
- 安装并启动 systemd 服务

```bash
#!/usr/bin/env bash
set -e

# 恢复配置文件
if [[ -f /opt/shortener-server/config/config.toml.bak ]]; then
  mv /opt/shortener-server/config/config.toml.bak /opt/shortener-server/config/config.toml
fi

# 创建默认配置
if [[ ! -f /opt/shortener-server/config/config.toml ]]; then
  mkdir -p /opt/shortener-server/config
  cp /opt/shortener-server/config.toml /opt/shortener-server/config/config.toml
fi

# 创建符号链接
if [[ -e /usr/local/bin/shortener-server ]]; then
  rm -rf /usr/local/bin/shortener-server
fi
ln -s /opt/shortener-server/shortener-server /usr/local/bin/shortener-server

# 安装 systemd 服务
if [[ -f /opt/shortener-server/shortener-server.service ]]; then
  cp /opt/shortener-server/shortener-server.service /etc/systemd/system/shortener-server.service
  systemctl enable shortener-server.service
  systemctl start shortener-server.service
fi

exit 0
```

### preremove.sh

卸载前执行的脚本，主要功能：
- 停止并禁用 systemd 服务

```bash
#!/usr/bin/env bash
set -e

if systemctl list-units --type=service | grep -q 'shortener-server.service'; then
  systemctl stop shortener-server.service
  systemctl disable shortener-server.service
fi

exit 0
```

### postremove.sh

卸载后执行的脚本，主要功能：
- 删除 systemd 服务文件
- 删除符号链接

```bash
#!/usr/bin/env bash
set -e

if [[ -f /etc/systemd/system/shortener-server.service ]]; then
  rm -rf /etc/systemd/system/shortener-server.service
fi

if [[ -e /usr/local/bin/shortener-server ]]; then
  rm -rf /usr/local/bin/shortener-server
fi

exit 0
```

## 使用 cargo-deb 打包

### 配置 Cargo.toml

在 `shortener-server/Cargo.toml` 中添加 deb 打包配置：

```toml
[package.metadata.deb]
maintainer = "Jetsung Chan <i@jetsung.com>"
copyright = "2025, Jetsung Chan <i@jetsung.com>"
license-file = ["../LICENSE", "0"]
extended-description = """\
A high-performance URL shortener service written in Rust, \
featuring a RESTful API server and command-line management tool."""
depends = "$auto, systemd"
section = "web"
priority = "optional"
assets = [
    ["target/release/shortener-server", "opt/shortener-server/", "755"],
    ["config/config.toml", "opt/shortener-server/config.toml", "644"],
    ["deploy/systemd/shortener-server.service", "opt/shortener-server/shortener-server.service", "644"],
    ["README.md", "usr/share/doc/shortener-server/README", "644"],
]
maintainer-scripts = "scripts/"
systemd-units = { enable = true }

[package.metadata.deb.variants.server]
name = "shortener-server"
```

### 构建 deb 包

```bash
# 构建 server 的 deb 包
cargo deb -p shortener-server

# 构建 CLI 的 deb 包
cargo deb -p shortener-cli

# 指定目标架构
cargo deb -p shortener-server --target=x86_64-unknown-linux-gnu

# 输出位置
# target/debian/shortener-server_<version>_<arch>.deb
```

## 手动打包

如果不使用 cargo-deb，可以手动创建 deb 包：

### 1. 创建包目录结构

```bash
mkdir -p shortener-server_0.3.0_amd64/DEBIAN
mkdir -p shortener-server_0.3.0_amd64/opt/shortener-server
mkdir -p shortener-server_0.3.0_amd64/opt/shortener-server/config
mkdir -p shortener-server_0.3.0_amd64/usr/share/doc/shortener-server
```

### 2. 复制文件

```bash
# 复制二进制文件
cp target/release/shortener-server shortener-server_0.3.0_amd64/opt/shortener-server/

# 复制配置文件
cp config/config.toml shortener-server_0.3.0_amd64/opt/shortener-server/config.toml

# 复制 systemd 服务文件
cp deploy/systemd/shortener-server.service shortener-server_0.3.0_amd64/opt/shortener-server/

# 复制文档
cp README.md shortener-server_0.3.0_amd64/usr/share/doc/shortener-server/
```

### 3. 创建 control 文件

创建 `shortener-server_0.3.0_amd64/DEBIAN/control`：

```
Package: shortener-server
Version: 0.3.0
Section: web
Priority: optional
Architecture: amd64
Depends: systemd
Maintainer: Jetsung Chan <i@jetsung.com>
Description: High-performance URL shortener service
 A high-performance URL shortener service written in Rust,
 featuring a RESTful API server and command-line management tool.
 .
 Supports multiple databases (SQLite, PostgreSQL, MySQL),
 Redis/Valkey caching, and GeoIP tracking.
```

### 4. 复制维护脚本

```bash
cp scripts/preinstall.sh shortener-server_0.3.0_amd64/DEBIAN/preinst
cp scripts/postinstall.sh shortener-server_0.3.0_amd64/DEBIAN/postinst
cp scripts/preremove.sh shortener-server_0.3.0_amd64/DEBIAN/prerm
cp scripts/postremove.sh shortener-server_0.3.0_amd64/DEBIAN/postrm

# 设置执行权限
chmod 755 shortener-server_0.3.0_amd64/DEBIAN/preinst
chmod 755 shortener-server_0.3.0_amd64/DEBIAN/postinst
chmod 755 shortener-server_0.3.0_amd64/DEBIAN/prerm
chmod 755 shortener-server_0.3.0_amd64/DEBIAN/postrm
```

### 5. 构建 deb 包

```bash
dpkg-deb --build shortener-server_0.3.0_amd64

# 输出: shortener-server_0.3.0_amd64.deb
```

### 6. 检查包内容

```bash
# 查看包信息
dpkg-deb --info shortener-server_0.3.0_amd64.deb

# 查看包内容
dpkg-deb --contents shortener-server_0.3.0_amd64.deb

# 检查包质量
lintian shortener-server_0.3.0_amd64.deb
```

## 安装和卸载

### 安装

```bash
# 使用 dpkg 安装
sudo dpkg -i shortener-server_0.3.0_amd64.deb

# 如果有依赖问题，使用 apt 修复
sudo apt-get install -f

# 或使用 apt 直接安装
sudo apt install ./shortener-server_0.3.0_amd64.deb
```

### 验证安装

```bash
# 检查服务状态
sudo systemctl status shortener-server

# 检查二进制文件
which shortener-server
shortener-server --version

# 检查配置文件
ls -la /opt/shortener-server/config/
```

### 卸载

```bash
# 卸载但保留配置文件
sudo dpkg -r shortener-server

# 完全卸载（包括配置文件）
sudo dpkg -P shortener-server

# 或使用 apt
sudo apt remove shortener-server
sudo apt purge shortener-server
```

## 测试

### 本地测试

```bash
# 1. 构建 deb 包
cargo deb -p shortener-server

# 2. 在测试环境安装
sudo dpkg -i target/debian/shortener-server_*.deb

# 3. 测试服务
sudo systemctl status shortener-server
curl http://localhost:8080/health

# 4. 测试 CLI
shortener-server --version

# 5. 卸载
sudo dpkg -r shortener-server
```

### Docker 测试

创建测试 Dockerfile：

```dockerfile
FROM debian:bookworm-slim

# 安装依赖
RUN apt-get update && apt-get install -y \
    systemd \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制 deb 包
COPY target/debian/shortener-server_*.deb /tmp/

# 安装
RUN dpkg -i /tmp/shortener-server_*.deb || apt-get install -f -y

# 测试
RUN shortener-server --version

CMD ["/lib/systemd/systemd"]
```

运行测试：

```bash
docker build -t shortener-deb-test -f Dockerfile.deb-test .
docker run --rm shortener-deb-test shortener-server --version
```

## 发布到 APT 仓库

### 使用 GitHub Releases

1. 构建 deb 包
2. 上传到 GitHub Releases
3. 用户可以直接下载安装

### 使用 PPA（Ubuntu）

1. 注册 Launchpad 账号
2. 创建 PPA
3. 上传源码包
4. Launchpad 自动构建

### 使用自建 APT 仓库

```bash
# 1. 创建仓库目录
mkdir -p /var/www/apt/pool/main

# 2. 复制 deb 包
cp shortener-server_*.deb /var/www/apt/pool/main/

# 3. 生成 Packages 文件
cd /var/www/apt
dpkg-scanpackages pool/main /dev/null | gzip -9c > pool/main/Packages.gz

# 4. 生成 Release 文件
apt-ftparchive release pool/main > pool/main/Release

# 5. 签名（可选）
gpg --default-key your-key-id -abs -o pool/main/Release.gpg pool/main/Release
```

用户添加仓库：

```bash
# 添加仓库
echo "deb [trusted=yes] https://your-domain.com/apt pool main" | sudo tee /etc/apt/sources.list.d/shortener.list

# 更新并安装
sudo apt update
sudo apt install shortener-server
```

## 最佳实践

1. **版本管理**: 使用语义化版本号
2. **依赖声明**: 在 control 文件中正确声明依赖
3. **配置保留**: 升级时保留用户配置
4. **服务管理**: 使用 systemd 管理服务
5. **权限设置**: 正确设置文件和目录权限
6. **文档完善**: 提供详细的安装和使用文档
7. **测试充分**: 在多个 Debian/Ubuntu 版本上测试
8. **签名验证**: 对包进行 GPG 签名

## 故障排查

### 安装失败

```bash
# 查看详细错误
sudo dpkg -i --debug=1000 shortener-server_*.deb

# 检查依赖
dpkg -I shortener-server_*.deb | grep Depends
apt-cache policy <dependency>
```

### 服务启动失败

```bash
# 查看服务日志
sudo journalctl -u shortener-server -n 50

# 检查配置文件
sudo cat /opt/shortener-server/config/config.toml

# 手动启动测试
sudo /opt/shortener-server/shortener-server
```

### 卸载残留

```bash
# 查找残留文件
dpkg -L shortener-server

# 手动清理
sudo rm -rf /opt/shortener-server
sudo rm -f /etc/systemd/system/shortener-server.service
sudo rm -f /usr/local/bin/shortener-server
```

## 相关资源

- [Debian 打包指南](https://www.debian.org/doc/manuals/maint-guide/)
- [cargo-deb 文档](https://github.com/kornelski/cargo-deb)
- [dpkg 手册](https://man7.org/linux/man-pages/man1/dpkg.1.html)
- [Debian 策略手册](https://www.debian.org/doc/debian-policy/)

## 相关文档

- [主文档](../README.md)
- [部署指南](DEPLOYMENT.md)
- [交叉编译指南](../scripts/CROSS_COMPILE.md)
