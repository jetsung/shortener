# 构建 DEB 包

本项目使用 `cargo-deb` 自动生成 Debian 安装包。

## 快速开始

### 1. 安装 cargo-deb

```bash
cargo install cargo-deb
```

### 2. 构建 deb 包

```bash
# 构建 server
cargo deb -p shortener-server

# 构建 CLI
cargo deb -p shortener-cli

# 输出位置
# target/debian/shortener-server_<version>_<arch>.deb
# target/debian/shortener-cli_<version>_<arch>.deb
```

### 3. 安装测试

```bash
# 安装
sudo dpkg -i target/debian/shortener-server_*.deb

# 或使用 apt（推荐，会自动处理依赖）
sudo apt install ./target/debian/shortener-server_*.deb

# 编辑配置
sudo nano /opt/shortener/config/config.toml

# 配置数据库路径（SQLite 示例）
# [database.sqlite]
# path = "/opt/shortener/data/shortener.db"

# 启动服务
sudo systemctl start shortener-server

# 查看状态
sudo systemctl status shortener-server
```

### 4. 卸载

```bash
# 卸载但保留配置
sudo dpkg -r shortener-server

# 完全卸载（包括配置和数据）
sudo dpkg -P shortener-server
```

## 配置说明

DEB 打包配置在 `shortener-server/Cargo.toml` 中：

```toml
[package.metadata.deb]
maintainer = "Jetsung Chan <i@jetsung.com>"
depends = "$auto, systemd"
assets = [
    ["target/release/shortener-server", "usr/local/bin/", "755"],
    ["../config/config.toml", "opt/shortener/config.toml.example", "644"],
    ["../deploy/systemd/shortener-server.service", "lib/systemd/system/", "644"],
    ["../README.md", "usr/share/doc/shortener-server/", "644"],
]
maintainer-scripts = "../scripts/"
```

## 安装后的文件布局

```
/usr/local/bin/shortener-server                # 二进制文件
/opt/shortener/config/config.toml              # 配置文件
/opt/shortener/config.toml.example             # 配置示例
/opt/shortener/data/                           # 数据目录（数据库、GeoIP 等）
/opt/shortener/logs/                           # 日志目录
/lib/systemd/system/shortener-server.service   # systemd 服务
```

## 维护脚本

项目包含 4 个维护脚本（位于 `scripts/` 目录）：

- `preinstall.sh`: 创建用户和组，备份配置
- `postinstall.sh`: 创建目录，设置权限，启用服务
- `preremove.sh`: 停止并禁用服务
- `postremove.sh`: 清理文件（仅在 purge 时）

## 详细文档

更多信息请参考：
- [DEB 打包完整指南](docs/DEB_PACKAGING.md)
- [DEB 打包简化指南](docs/DEB_PACKAGING_SIMPLIFIED.md)
