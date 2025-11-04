# GeoIP 配置指南

本指南介绍如何在 Shortener 中启用和配置 GeoIP 功能，用于追踪访问者的地理位置信息。

## 目录

- [概述](#概述)
- [ip2region 简介](#ip2region-简介)
- [安装步骤](#安装步骤)
- [配置说明](#配置说明)
- [使用示例](#使用示例)
- [性能优化](#性能优化)
- [故障排除](#故障排除)

## 概述

Shortener 支持使用 GeoIP 功能来追踪短链接访问者的地理位置信息。目前支持的 GeoIP 提供商：

- **ip2region** - 高性能的离线 IP 地址定位库（推荐）

GeoIP 功能默认是**禁用**的，需要手动下载数据库文件并配置后才能使用。

## ip2region 简介

[ip2region](https://github.com/lionsoul2014/ip2region) 是一个离线 IP 地址定位库和 IP 定位数据管理框架，具有以下特点：

- **高性能**：10 微秒级别的查询速度
- **准确性高**：提供国家、省份、城市级别的定位
- **体积小**：数据库文件仅约 11MB
- **离线使用**：无需网络请求，保护隐私
- **多语言支持**：支持多种编程语言

### 数据格式

ip2region 返回的地理位置信息格式：

```
国家|区域|省份|城市|ISP
```

示例：
```
中国|0|浙江省|杭州市|阿里云
美国|0|加利福尼亚|洛杉矶|0
```

## 安装步骤

### 1. 下载 ip2region 数据库

从 GitHub 下载最新的 ip2region 数据库文件：

```bash
# 创建数据目录
mkdir -p data

# 下载 IPv4 数据库（推荐）
curl -fsSL https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb \
    -o data/ip2region.xdb

# 或者使用 wget
wget https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb \
    -O data/ip2region.xdb
```

**数据库文件说明：**
- `ip2region_v4.xdb` - IPv4 数据库（约 11MB）
- 数据库会定期更新，建议定期下载最新版本

### 2. 验证文件

确认文件已成功下载：

```bash
ls -lh data/ip2region.xdb
```

应该看到类似输出：
```
-rw-r--r-- 1 user user 11M Nov 4 2025 data/ip2region.xdb
```

### 3. 配置 Shortener

编辑配置文件 `config/config.toml`：

```toml
[geoip]
# 启用 GeoIP 功能
enabled = true

# 使用 ip2region 提供商
type = "ip2region"

[geoip.ip2region]
# 数据库文件路径
path = "data/ip2region.xdb"

# 搜索模式：vector（最快）、btree（平衡）、binary（最小内存）
mode = "vector"

# IP 版本：4 表示 IPv4
version = "4"
```

### 4. 重启服务

```bash
# 如果使用 systemd
sudo systemctl restart shortener

# 如果直接运行
cargo run --release -p shortener-server

# 如果使用 Docker
docker compose restart
```

## 配置说明

### 搜索模式

ip2region 支持三种搜索模式，各有优缺点：

| 模式 | 速度 | 内存占用 | 适用场景 |
|------|------|----------|----------|
| `vector` | 最快（~10μs） | 最大（~11MB） | 生产环境（推荐） |
| `btree` | 中等（~20μs） | 中等（~5MB） | 内存受限环境 |
| `binary` | 较慢（~50μs） | 最小（~1MB） | 极度内存受限 |

**推荐配置：**
```toml
[geoip.ip2region]
mode = "vector"  # 生产环境推荐
```

### 文件路径

根据部署方式配置不同的路径：

**本地开发：**
```toml
path = "data/ip2region.xdb"
```

**Docker 部署：**
```toml
path = "/var/lib/shortener/ip2region.xdb"
```

**系统服务：**
```toml
path = "/var/lib/shortener/ip2region.xdb"
```

## 使用示例

### Docker 部署

如果使用 Docker 部署，需要将数据库文件挂载到容器中：

#### 方法 1：使用 Volume 挂载

```yaml
# docker-compose.yml
services:
  shortener:
    image: ghcr.io/jetsung/shortener:latest
    volumes:
      - ./data/ip2region.xdb:/var/lib/shortener/ip2region.xdb:ro
    environment:
      - GEOIP_ENABLED=true
```

#### 方法 2：在容器启动时下载

```dockerfile
# 自定义 Dockerfile
FROM ghcr.io/jetsung/shortener:latest

USER root

# 下载 ip2region 数据库
RUN apt-get update && apt-get install -y curl && \
    curl -fsSL https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb \
    -o /var/lib/shortener/ip2region.xdb && \
    chown shortener:shortener /var/lib/shortener/ip2region.xdb && \
    apt-get remove -y curl && \
    rm -rf /var/lib/apt/lists/*

USER shortener
```

### 环境变量配置

使用环境变量启用 GeoIP：

```bash
# 启用 GeoIP
export SHORTENER__GEOIP__ENABLED=true

# 设置数据库路径
export SHORTENER__GEOIP__IP2REGION__PATH=/path/to/ip2region.xdb

# 设置搜索模式
export SHORTENER__GEOIP__IP2REGION__MODE=vector
```

### 验证配置

启动服务后，检查日志确认 GeoIP 已启用：

```bash
# 查看日志
tail -f /var/log/shortener/shortener.log

# 应该看到类似输出
[INFO] GeoIP enabled with provider: ip2region
[INFO] ip2region database loaded: data/ip2region.xdb
```

## 性能优化

### 1. 选择合适的搜索模式

生产环境推荐使用 `vector` 模式以获得最佳性能：

```toml
[geoip.ip2region]
mode = "vector"
```

### 2. 启用缓存

配合 Redis 缓存可以进一步提升性能：

```toml
[cache]
enabled = true
type = "redis"
expire = 3600  # 缓存 1 小时

[cache.redis]
host = "localhost"
port = 6379
```

### 3. 定期更新数据库

ip2region 数据库会定期更新，建议每月更新一次：

```bash
#!/bin/bash
# update-geoip.sh

# 备份旧数据库
cp data/ip2region.xdb data/ip2region.xdb.bak

# 下载新数据库
curl -fsSL https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb \
    -o data/ip2region.xdb

# 重启服务
systemctl restart shortener

echo "GeoIP database updated successfully"
```

设置定时任务：
```bash
# 每月 1 号凌晨 2 点更新
0 2 1 * * /path/to/update-geoip.sh
```

## 故障排除

### 问题 1：数据库文件未找到

**错误信息：**
```
Error: Failed to load ip2region database: No such file or directory
```

**解决方法：**
1. 确认文件路径正确
2. 检查文件权限
3. 确认文件已下载完整

```bash
# 检查文件
ls -lh data/ip2region.xdb

# 检查权限
chmod 644 data/ip2region.xdb
```

### 问题 2：GeoIP 查询失败

**错误信息：**
```
Warning: GeoIP lookup failed for IP: xxx.xxx.xxx.xxx
```

**可能原因：**
1. 数据库文件损坏
2. IP 地址格式不正确
3. 数据库版本不匹配

**解决方法：**
```bash
# 重新下载数据库
rm data/ip2region.xdb
curl -fsSL https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb \
    -o data/ip2region.xdb
```

### 问题 3：内存占用过高

**解决方法：**

切换到更节省内存的搜索模式：

```toml
[geoip.ip2region]
mode = "btree"  # 或 "binary"
```

### 问题 4：Docker 容器中无法访问文件

**解决方法：**

确保文件已正确挂载：

```bash
# 检查容器内文件
docker exec shortener ls -lh /var/lib/shortener/ip2region.xdb

# 如果文件不存在，检查 volume 挂载
docker inspect shortener | grep -A 10 Mounts
```

## 相关资源

- [ip2region GitHub 仓库](https://github.com/lionsoul2014/ip2region)
- [ip2region 文档](https://github.com/lionsoul2014/ip2region/blob/master/README.md)
- [数据库下载地址](https://github.com/lionsoul2014/ip2region/tree/master/data)
- [Shortener 配置指南](CONFIGURATION.md)
- [Docker 部署指南](../deployment/DOCKER.md)

## 另见

- [配置指南](CONFIGURATION.md)
- [API 文档](../server/API.md)
- [部署指南](../deployment/DEPLOYMENT.md)
