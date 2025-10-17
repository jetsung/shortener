# 部署指南

在各种环境中部署 Shortener 服务器的完整指南。

## 目录

- [概述](#概述)
- [前提条件](#前提条件)
- [部署方法](#部署方法)
- [Docker 部署](#docker-部署)
- [Systemd 部署](#systemd-部署)
- [手动部署](#手动部署)
- [云部署](#云部署)
- [反向代理设置](#反向代理设置)
- [SSL/TLS 配置](#ssltls-配置)
- [监控](#监控)
- [备份和恢复](#备份和恢复)

## 概述

Shortener 服务器可以通过多种方式部署：

1. Docker - 推荐用于大多数场景
2. Systemd - 用于传统 Linux 部署
3. 手动 - 用于自定义设置
4. 云 - AWS、GCP、Azure 等

## 前提条件

### 系统要求

- CPU：最少 1 核，推荐 2+ 核
- 内存：最少 512MB，推荐 1GB+
- 磁盘：最少 1GB，推荐 10GB+
- 操作系统：Linux（Ubuntu 20.04+、Debian 11+、CentOS 8+ 等）

### 软件要求

- Rust：1.85+（从源码构建）
- Docker：20.10+（Docker 部署）
- PostgreSQL：12+ 或 MySQL：8.0+（可选，生产环境）
- Redis：6.0+ 或 Valkey：7.0+（可选，缓存）

## 部署方法

### 快速比较

| 方法 | 难度 | 隔离性 | 可移植性 | 最适合 |
|------|------|--------|----------|--------|
| Docker | 简单 | 高 | 高 | 大多数部署 |
| Systemd | 中等 | 低 | 低 | 传统 Linux |
| 手动 | 困难 | 低 | 低 | 自定义设置 |
| 云 | 简单 | 高 | 高 | 可扩展部署 |

## Docker 部署

Docker 是推荐的部署方式，提供了简单的设置和良好的隔离性。

详细的 Docker 部署说明请参阅 **[Docker 部署指南](DOCKER.md)**，包括：

- 开发和生产环境配置
- Docker Compose 设置
- 多数据库支持
- 环境变量配置
- 数据持久化
- 健康检查和监控

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 使用 Docker Compose 启动
docker compose up -d
```

更多详细信息请查看 [Docker 部署指南](DOCKER.md)。
ADMIN_PASSWORD=$(openssl rand -base64 32)
EOF
```

3. 启动服务：

```bash
docker compose --env-file .env up -d
```

4. 验证部署：

```bash
# 检查容器状态
docker compose ps

# 检查日志
docker compose logs -f shortener-server

# 测试 API
curl http://localhost:8080/health
```

详细说明请参阅 [Docker 部署指南](DOCKER.md)。

## Systemd 部署

### 安装

1. 构建二进制文件：

```bash
cargo build --release -p shortener-server
```

2. 安装 DEB 包（推荐）：

```bash
# 下载并安装 DEB 包
sudo apt install ./shortener-server_*.deb
```

或手动安装：

```bash
cd deploy/systemd
sudo ./install.sh
```

这将：
- 复制二进制文件到 `/usr/local/bin/`
- 创建配置目录 `/opt/shortener/config/`
- 创建数据目录 `/opt/shortener/data/`
- 创建日志目录 `/opt/shortener/logs/`
- 创建 systemd 服务文件
- 创建 `shortener` 用户和组

3. 配置服务：

```bash
sudo vim /opt/shortener/config/config.toml
```

配置数据库路径：

```toml
[database.sqlite]
path = "/opt/shortener/data/shortener.db"
```

4. 启动服务：

```bash
sudo systemctl start shortener-server
sudo systemctl enable shortener-server
```

5. 验证：

```bash
sudo systemctl status shortener-server
sudo journalctl -u shortener-server -f
```

### 服务管理

```bash
# 启动
sudo systemctl start shortener-server

# 停止
sudo systemctl stop shortener-server

# 重启
sudo systemctl restart shortener-server

# 状态
sudo systemctl status shortener-server

# 启用自动启动
sudo systemctl enable shortener-server

# 查看日志
sudo journalctl -u shortener-server -f
```

## 手动部署

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 构建发布版本
cargo build --release -p shortener-server

# 二进制文件位置
ls -lh target/release/shortener-server
```

### 设置目录

```bash
# 创建目录
sudo mkdir -p /opt/shortener/config
sudo mkdir -p /opt/shortener/data
sudo mkdir -p /opt/shortener/logs

# 复制文件
sudo cp target/release/shortener-server /usr/local/bin/
sudo cp config/config.toml /opt/shortener/config/

# 设置权限
sudo chown -R shortener:shortener /opt/shortener
```

### 创建用户

```bash
sudo useradd -r -s /bin/false shortener
```

### 手动运行

```bash
# 以 shortener 用户运行
sudo -u shortener /usr/local/bin/shortener-server

# 或使用自定义配置
sudo -u shortener /usr/local/bin/shortener-server --config /opt/shortener/config/config.toml
```

## 云部署

### AWS

#### EC2 部署

1. 启动 EC2 实例（Ubuntu 22.04 LTS）
2. 安装 Docker：

```bash
sudo apt update
sudo apt install -y docker.io docker-compose-plugin
sudo usermod -aG docker ubuntu
```

3. 部署应用：

```bash
git clone https://github.com/jetsung/shortener.git
cd shortener
docker compose up -d
```

#### RDS 数据库

```toml
[database]
type = "postgres"

[database.postgres]
host = "shortener.xxxxx.us-east-1.rds.amazonaws.com"
port = 5432
user = "shortener"
password = "${DB_PASSWORD}"
database = "shortener"
sslmode = "require"
```

#### ElastiCache Redis

```toml
[cache]
enabled = true
type = "redis"

[cache.redis]
host = "shortener.xxxxx.cache.amazonaws.com"
port = 6379
password = "${REDIS_PASSWORD}"
```

### Google Cloud Platform

#### Compute Engine

类似于 AWS EC2 部署。

#### Cloud Run

1. 构建容器：

```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/shortener-server
```

2. 部署到 Cloud Run：

```bash
gcloud run deploy shortener-server \
  --image gcr.io/PROJECT_ID/shortener-server \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

## 反向代理设置

### Nginx

```nginx
upstream shortener {
    server localhost:8080;
}

server {
    listen 80;
    server_name short.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name short.example.com;

    ssl_certificate /etc/letsencrypt/live/short.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/short.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    location / {
        proxy_pass http://shortener;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

```caddyfile
short.example.com {
    reverse_proxy localhost:8080
}
```

## SSL/TLS 配置

### Let's Encrypt 与 Certbot

```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d short.example.com

# 自动续期
sudo certbot renew --dry-run
```

## 监控

### 健康检查

```bash
# 检查服务器是否运行
curl http://localhost:8080/health
```

### 日志

```bash
# 查看日志（Docker）
docker compose logs -f shortener-server

# 查看日志（Systemd）
sudo journalctl -u shortener-server -f

# 查看日志（手动）
tail -f /var/log/shortener/shortener.log
```

## 备份和恢复

### 数据库备份

#### PostgreSQL

```bash
# 备份
pg_dump -U shortener -h localhost shortener > backup.sql

# 恢复
psql -U shortener -h localhost shortener < backup.sql
```

#### MySQL

```bash
# 备份
mysqldump -u shortener -p shortener > backup.sql

# 恢复
mysql -u shortener -p shortener < backup.sql
```

#### SQLite

```bash
# 备份
sqlite3 /opt/shortener/data/shortener.db ".backup backup.db"

# 恢复
cp backup.db /opt/shortener/data/shortener.db
sudo chown shortener:shortener /opt/shortener/data/shortener.db
```

### 配置和数据备份

```bash
# 备份整个 shortener 目录
sudo tar czf shortener-backup-$(date +%Y%m%d).tar.gz /opt/shortener/

# 恢复
sudo tar xzf shortener-backup-20250117.tar.gz -C /
sudo chown -R shortener:shortener /opt/shortener
```

## 故障排除

### 服务器无法启动

1. 检查配置：
```bash
shortener-server --config /opt/shortener/config/config.toml --check
```

2. 检查日志：
```bash
sudo journalctl -u shortener-server -n 50
```

3. 检查端口可用性：
```bash
sudo lsof -i :8080
```

### 数据库连接问题

1. 测试连接：
```bash
# PostgreSQL
psql -U shortener -h localhost -d shortener

# MySQL
mysql -u shortener -h localhost -p shortener
```

2. 检查防火墙：
```bash
sudo ufw status
```

### 缓存连接问题

1. 测试 Redis 连接：
```bash
redis-cli -h localhost -p 6379 ping
```

2. 检查 Redis 日志：
```bash
sudo journalctl -u redis -f
```

## 最佳实践

1. 生产环境使用 HTTPS
2. 启用缓存以获得更好的性能
3. 定期备份数据库和配置
4. 监控服务器健康和性能
5. 使用环境变量存储密钥
6. 保持软件更新（安全补丁）
7. 使用反向代理（Nginx/Caddy）
8. 设置日志轮转
9. 正确配置防火墙
10. 测试灾难恢复程序

## 安全检查清单

- [ ] 启用 HTTPS 并使用有效证书
- [ ] 使用强 API 密钥和管理员密码
- [ ] 配置防火墙（仅开放必要端口）
- [ ] 数据库不暴露到互联网
- [ ] Redis/Valkey 密码保护
- [ ] 定期安全更新
- [ ] 启用日志监控
- [ ] 启用备份加密
- [ ] 配置速率限制（反向代理）
- [ ] 配置安全头（反向代理）

## 另见

- [Docker 部署](DOCKER.md)
- [DEB 包安装](DEB_PACKAGING_SIMPLIFIED.md)
- [配置指南](CONFIGURATION.md)
- [API 文档](API.md)
