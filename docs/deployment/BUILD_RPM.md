# 构建 RPM 包

本项目使用 `cargo-generate-rpm` 自动生成 RPM 安装包，支持 RHEL / CentOS /
Fedora / openSUSE / Rocky Linux / AlmaLinux 等 RPM 系发行版。

## 快速开始

### 1. 安装 cargo-generate-rpm

```bash
cargo install cargo-generate-rpm
```

### 2. 构建 rpm 包

```bash
# 先构建 release 二进制（如尚未构建）
cargo build --release -p shortener-server

# 构建 rpm
cargo generate-rpm -p shortener-server

# 输出位置
# target/generate-rpm/shortener-server_<version>-<release>.<arch>.rpm
```

### 3. 安装测试

```bash
# 安装
sudo rpm -i target/generate-rpm/shortener-server-*.rpm

# 或使用 dnf/yum（推荐，会自动处理依赖）
sudo dnf install ./target/generate-rpm/shortener-server-*.rpm

# 编辑配置
sudo nano /opt/shortener/config/config.toml

# 配置数据库路径（SQLite 示例）
# [database]
# url = "sqlite:///opt/shortener/data/shortener.db?mode=rwc"

# 启动服务
sudo systemctl start shortener-server

# 查看状态
sudo systemctl status shortener-server
```

### 4. 卸载

```bash
# 卸载（prerm 脚本会停止并禁用服务，配置文件保留）
sudo rpm -e shortener-server
```

## 配置说明

RPM 打包配置在 `shortener-server/Cargo.toml` 中，资产与 DEB 包保持一致：

```toml
[package.metadata.generate-rpm]
vendor = "Jetsung Chan <i@jetsung.com>"
release = "1"
summary = "High-performance URL shortener service written in Rust"
pre_install_script = "../scripts/preinst"
post_install_script = "../scripts/postinst"
pre_uninstall_script = "../scripts/prerm"
requires = { systemd = "" }

[[package.metadata.generate-rpm.assets]]
source = "target/release/shortener-server"
dest = "/usr/local/bin/shortener-server"
mode = "755"

[[package.metadata.generate-rpm.assets]]
source = "../config.toml"
dest = "/opt/shortener/config.toml.example"
mode = "644"
config = true

[[package.metadata.generate-rpm.assets]]
source = "../deploy/systemd/shortener-server.service"
dest = "/lib/systemd/system/shortener-server.service"
mode = "644"

[[package.metadata.generate-rpm.assets]]
source = "../README.md"
dest = "/usr/share/doc/shortener-server/README.md"
mode = "644"
doc = true
```

字段说明：

- `vendor`：包维护者（RPM 没有 deb 的 `maintainer` 字段，用 `vendor` 承载同一信息）
- `release`：RPM 的 release 字段，独立于版本号
- `requires`：显式依赖；动态库依赖（GLIBC、OpenSSL 等）由 `--auto-req` 自动检测
- 资产项的 `config = true` 对应 deb 的 `conffiles`（升级时不覆盖用户修改的配置文件），
  `doc = true` 对应 deb 的文档文件标记

## 安装后的文件布局

与 DEB 包一致：

```
/usr/local/bin/shortener-server                # 二进制文件
/opt/shortener/config/config.toml              # 配置文件（postinst 创建）
/opt/shortener/config.toml.example             # 配置示例
/opt/shortener/data/                           # 数据目录（数据库、GeoIP 等）
/opt/shortener/logs/                           # 日志目录
/lib/systemd/system/shortener-server.service   # systemd 服务
/usr/share/doc/shortener-server/README.md      # 文档
```

## 维护脚本

RPM 通过 `*_script` 字段直接引用 `scripts/` 目录下的脚本（同一批文件也供
DEB 包使用）：

- `preinst`：创建用户和组，备份配置
- `postinst`：创建目录，设置权限，启用服务
- `prerm`：停止并禁用服务

## 详细文档

更多信息请参考：
- [DEB 包构建](BUILD_DEB.md)
- [DEB 打包指南](DEB_PACKAGING_SIMPLIFIED.md)
