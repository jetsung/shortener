# DEB 包安装指南

本文档介绍如何使用 DEB 包安装和管理 Shortener 服务。

## 快速安装

### 1. 下载 DEB 包

从 [GitHub Releases](https://github.com/jetsung/shortener/releases) 下载最新的 DEB 包。

### 2. 安装

```bash
# 使用 apt 安装（推荐，会自动处理依赖）
sudo apt install ./shortener-server_*.deb

# 或使用 dpkg
sudo dpkg -i shortener-server_*.deb
```

### 3. 配置

编辑配置文件：

```bash
sudo nano /opt/shortener/config/config.toml
```

重要配置项：

```toml
[server]
address = ":8080"
site_url = "http://your-domain.com"
api_key = "your-secret-key"

[database]
type = "sqlite"

[database.sqlite]
path = "/opt/shortener/data/shortener.db"

[geoip]
enabled = true
type = "ip2region"
# 数据文件路径：/opt/shortener/data/ip2region.xdb
```

### 4. 启动服务

```bash
# 启动服务
sudo systemctl start shortener-server

# 查看状态
sudo systemctl status shortener-server

# 查看日志
sudo journalctl -u shortener-server -f
```

## 文件布局

安装后的文件结构：

```
/usr/local/bin/shortener-server                # 二进制文件
/opt/shortener/
├── config/
│   ├── config.toml                            # 配置文件
│   └── config.toml.example                    # 配置示例
├── data/                                      # 数据目录
│   ├── shortener.db                           # SQLite 数据库
│   └── ip2region.xdb                          # GeoIP 数据文件
└── logs/                                      # 日志目录
/lib/systemd/system/shortener-server.service   # systemd 服务
/usr/share/doc/shortener-server/README         # 文档
```

## 数据文件说明

### 数据库文件

如果使用 SQLite，数据库文件默认保存在：

```
/opt/shortener/data/shortener.db
```

配置文件中设置：

```toml
[database.sqlite]
path = "/opt/shortener/data/shortener.db"
```

### GeoIP 数据文件

如果启用 GeoIP 功能，需要将 `ip2region.xdb` 文件放在：

```
/opt/shortener/data/ip2region.xdb
```

下载地址：[ip2region](https://github.com/lionsoul2014/ip2region)

配置文件中设置：

```toml
[geoip]
enabled = true
type = "ip2region"
```

### 日志文件

日志文件保存在：

```
/opt/shortener/logs/
```

可以在配置文件中自定义日志设置：

```toml
[logging]
level = "info"
format = "json"
```

## 服务管理

### 启动/停止服务

```bash
# 启动
sudo systemctl start shortener-server

# 停止
sudo systemctl stop shortener-server

# 重启
sudo systemctl restart shortener-server

# 查看状态
sudo systemctl status shortener-server
```

### 开机自启

```bash
# 启用开机自启（安装时已自动启用）
sudo systemctl enable shortener-server

# 禁用开机自启
sudo systemctl disable shortener-server
```

### 查看日志

```bash
# 实时查看日志
sudo journalctl -u shortener-server -f

# 查看最近 50 条日志
sudo journalctl -u shortener-server -n 50

# 查看今天的日志
sudo journalctl -u shortener-server --since today
```

## 权限说明

服务以 `shortener` 用户运行，所有文件和目录都属于该用户：

```bash
# 查看权限
ls -la /opt/shortener/

# 如需修改权限
sudo chown -R shortener:shortener /opt/shortener
sudo chmod 640 /opt/shortener/config/config.toml
sudo chmod 755 /opt/shortener/data
sudo chmod 755 /opt/shortener/logs
```

## 升级

直接安装新版本的 DEB 包即可，配置文件和数据会自动保留：

```bash
sudo apt install ./shortener-server_<new-version>_amd64.deb
```

服务会自动重启。

## 卸载

### 卸载但保留配置和数据

```bash
sudo dpkg -r shortener-server
```

### 完全卸载（包括配置和数据）

```bash
sudo dpkg -P shortener-server
```

!!! warning "警告"
    完全卸载会删除所有数据，包括数据库文件和配置文件，请提前备份！

## 备份和恢复

### 备份

```bash
# 停止服务
sudo systemctl stop shortener-server

# 备份配置和数据
sudo tar czf shortener-backup-$(date +%Y%m%d).tar.gz \
    /opt/shortener/config/ \
    /opt/shortener/data/

# 启动服务
sudo systemctl start shortener-server
```

### 恢复

```bash
# 停止服务
sudo systemctl stop shortener-server

# 恢复数据
sudo tar xzf shortener-backup-20250117.tar.gz -C /

# 设置权限
sudo chown -R shortener:shortener /opt/shortener

# 启动服务
sudo systemctl start shortener-server
```

## 故障排查

### 服务启动失败

```bash
# 查看详细日志
sudo journalctl -u shortener-server -n 50

# 检查配置文件
sudo cat /opt/shortener/config/config.toml

# 检查数据目录权限
ls -la /opt/shortener/data/

# 手动测试
sudo -u shortener /usr/local/bin/shortener-server
```

### 数据库连接失败

检查数据库文件路径和权限：

```bash
# 检查文件是否存在
ls -la /opt/shortener/data/shortener.db

# 检查权限
sudo chown shortener:shortener /opt/shortener/data/shortener.db
sudo chmod 644 /opt/shortener/data/shortener.db
```

### GeoIP 功能不工作

检查 GeoIP 数据文件：

```bash
# 检查文件是否存在
ls -la /opt/shortener/data/ip2region.xdb

# 下载数据文件
cd /opt/shortener/data/
sudo wget https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region.xdb
sudo chown shortener:shortener ip2region.xdb
```

### 端口被占用

```bash
# 检查端口占用
sudo netstat -tlnp | grep 8080

# 或使用 ss
sudo ss -tlnp | grep 8080

# 修改配置文件中的端口
sudo nano /opt/shortener/config/config.toml
```

## 从源码构建 DEB 包

如果需要自己构建 DEB 包：

```bash
# 安装 cargo-deb
cargo install cargo-deb

# 构建
cargo deb -p shortener-server

# 输出位置
# target/debian/shortener-server_<version>_<arch>.deb
```

详细信息请参考 [DEB 打包完整指南](DEB_PACKAGING.md)。

## 相关文档

- [配置指南](CONFIGURATION.md)
- [部署指南](DEPLOYMENT.md)
- [API 文档](API.md)
- [DEB 打包完整指南](DEB_PACKAGING.md)
