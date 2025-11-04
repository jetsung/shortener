# 部署文档

本目录包含 Shortener 项目的部署相关文档。

## 文档列表

- [部署指南](DEPLOYMENT.md) - 生产环境部署最佳实践
- [Docker 部署](DOCKER.md) - 使用 Docker 和 Docker Compose 部署
- [DEB 包安装](DEB_PACKAGING_SIMPLIFIED.md) - Debian/Ubuntu 系统安装
- [DEB 包构建](BUILD_DEB.md) - 如何构建 Debian 包

## 快速开始

### Docker 部署（推荐）

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 使用 Docker Compose 启动
docker-compose up -d
```

详见：[Docker 部署](DOCKER.md)

### 传统部署

1. 编译项目
2. 配置数据库
3. 配置服务
4. 启动服务

详见：[部署指南](DEPLOYMENT.md)

### DEB 包安装

```bash
# 下载 DEB 包
wget https://github.com/jetsung/shortener/releases/latest/download/shortener_amd64.deb

# 安装
sudo dpkg -i shortener_amd64.deb
```

详见：[DEB 包安装](DEB_PACKAGING_SIMPLIFIED.md)

## 部署架构

### 单机部署
- 应用服务器
- SQLite 数据库
- 本地缓存

### 分布式部署
- 多个应用服务器（负载均衡）
- PostgreSQL/MySQL 数据库
- Redis/Valkey 缓存集群

## 相关文档

- [配置说明](../general/CONFIGURATION.md)
- [安装指南](../general/INSTALLATION.md)
- [服务器文档](../server/README.md)
