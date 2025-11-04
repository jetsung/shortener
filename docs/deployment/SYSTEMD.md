# Systemd 服务部署

此目录包含在 Linux 系统上部署 Shortener 服务器的 systemd 服务配置。

## 快速开始

### 安装

```bash
# 首先构建发布版本
cd ../..
cargo build --release -p shortener-server

# 安装服务（需要 root 权限）
cd deploy/systemd
sudo ./install.sh
```

### 配置

编辑配置文件：

```bash
sudo vim /etc/shortener/config.toml
```

### 启动服务

```bash
# 启动服务
sudo systemctl start shortener-server

# 启用开机自启动
sudo systemctl enable shortener-server

# 检查状态
sudo systemctl status shortener-server
```

## 服务管理

### 启动/停止/重启

```bash
# 启动
sudo systemctl start shortener-server

# 停止
sudo systemctl stop shortener-server

# 重启
sudo systemctl restart shortener-server

# 重新加载配置（如果支持）
sudo systemctl reload shortener-server
```

### 启用/禁用自动启动

```bash
# 启用开机自启动
sudo systemctl enable shortener-server

# 禁用自动启动
sudo systemctl disable shortener-server

# 检查是否已启用
systemctl is-enabled shortener-server
```

### 查看状态

```bash
# 详细状态
sudo systemctl status shortener-server

# 检查是否运行
systemctl is-active shortener-server

# 检查是否失败
systemctl is-failed shortener-server
```

## 日志

### 查看日志

```bash
# 查看最近的日志
sudo journalctl -u shortener-server

# 实时跟踪日志
sudo journalctl -u shortener-server -f

# 查看自启动以来的日志
sudo journalctl -u shortener-server -b

# 查看最近一小时的日志
sudo journalctl -u shortener-server --since "1 hour ago"

# 查看特定优先级的日志
sudo journalctl -u shortener-server -p err

# 导出日志到文件
sudo journalctl -u shortener-server > shortener.log
```

### 日志轮转

Systemd 通过 journald 自动处理日志轮转。在 `/etc/systemd/journald.conf` 中配置：

```ini
[Journal]
SystemMaxUse=500M
SystemMaxFileSize=100M
SystemMaxFiles=5
```

## 文件位置

### 二进制文件
- `/usr/local/bin/shortener-server` - 主可执行文件

### 配置
- `/etc/shortener/config.toml` - 主配置文件

### 数据
- `/var/lib/shortener/` - 数据目录
  - `shortener.db` - SQLite 数据库（如果使用 SQLite）
  - `ip2region.xdb` - GeoIP 数据库

### 日志
- `/var/log/shortener/` - 日志目录（如果启用文件日志）
- `journalctl -u shortener-server` - Systemd 日志

### 服务
- `/etc/systemd/system/shortener-server.service` - 服务单元文件

## 服务配置

服务文件包括：

### 安全特性

- **NoNewPrivileges**：防止权限提升
- **PrivateTmp**：隔离的 /tmp 目录
- **ProtectSystem**：只读系统目录
- **ProtectHome**：无法访问家目录
- **ProtectKernelTunables**：受保护的内核参数
- **RestrictRealtime**：无实时调度
- **MemoryDenyWriteExecute**：W^X 内存保护

### 资源限制

- **LimitNOFILE**：65536 个打开文件
- **LimitNPROC**：512 个进程

### 重启策略

- **Restart**：失败时重启
- **RestartSec**：5 秒
- **StartLimitInterval**：60 秒
- **StartLimitBurst**：3 次尝试

## 自定义

### 编辑服务文件

```bash
# 编辑服务文件
sudo systemctl edit --full shortener-server

# 或直接编辑
sudo vim /etc/systemd/system/shortener-server.service

# 更改后重新加载
sudo systemctl daemon-reload
sudo systemctl restart shortener-server
```

### 环境变量

在服务文件中添加环境变量：

```ini
[Service]
Environment="RUST_LOG=debug"
Environment="DATABASE_TYPE=postgres"
Environment="DATABASE_HOST=localhost"
```

或使用环境文件：

```ini
[Service]
EnvironmentFile=/etc/shortener/environment
```

创建 `/etc/shortener/environment`：

```bash
RUST_LOG=info
DATABASE_TYPE=postgres
DATABASE_HOST=localhost
DATABASE_PORT=5432
```

## 故障排除

### 服务无法启动

```bash
# 检查状态
sudo systemctl status shortener-server

# 查看详细日志
sudo journalctl -u shortener-server -n 50 --no-pager

# 检查配置
sudo /usr/local/bin/shortener-server --version

# 测试配置
sudo -u shortener /usr/local/bin/shortener-server
```

### 权限问题

```bash
# 检查文件所有权
ls -la /usr/local/bin/shortener-server
ls -la /etc/shortener/config.toml
ls -la /var/lib/shortener/

# 修复所有权
sudo chown shortener:shortener /var/lib/shortener/
sudo chown shortener:shortener /var/log/shortener/
```

### 端口已被占用

```bash
# 检查什么在使用端口 8080
sudo lsof -i :8080
sudo netstat -tulpn | grep 8080

# 在配置中更改端口
sudo vim /etc/shortener/config.toml
```

### 服务立即崩溃

```bash
# 检查错误
sudo journalctl -u shortener-server -n 100 --no-pager

# 手动运行以查看错误
sudo -u shortener /usr/local/bin/shortener-server

# 检查依赖项
ldd /usr/local/bin/shortener-server
```

## 卸载

```bash
# 运行卸载脚本
cd deploy/systemd
sudo ./uninstall.sh

# 手动删除所有文件（可选）
sudo rm -rf /usr/local/bin/shortener-server
sudo rm -rf /etc/shortener
sudo rm -rf /var/lib/shortener
sudo rm -rf /var/log/shortener
sudo userdel shortener
```

## 与其他服务集成

### Nginx 反向代理

创建 `/etc/nginx/sites-available/shortener`：

```nginx
server {
    listen 80;
    server_name short.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

启用并重启：

```bash
sudo ln -s /etc/nginx/sites-available/shortener /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### PostgreSQL 数据库

确保 PostgreSQL 在服务之前启动：

```ini
[Unit]
After=network-online.target postgresql.service
Wants=network-online.target
Requires=postgresql.service
```

### Redis 缓存

确保 Redis 在服务之前启动：

```ini
[Unit]
After=network-online.target redis.service
Wants=network-online.target
Requires=redis.service
```

## 监控

### Systemd 状态

```bash
# 监视状态
watch -n 1 'systemctl status shortener-server'

# 检查重启次数
systemctl show shortener-server -p NRestarts
```

### 资源使用

```bash
# CPU 和内存
systemd-cgtop

# 详细统计
systemctl show shortener-server
```

## 最佳实践

1. **始终测试配置**，然后再在生产环境中重启
2. **使用环境文件**存储敏感数据
3. **启用自动启动**用于生产服务
4. **定期监控日志**
5. **设置资源限制**以防止资源耗尽
6. **使用 systemd 提供的安全特性**
7. **保留配置和数据的备份**
8. **记录服务配置的更改**

## 参考

- [systemd.service 手册](https://www.freedesktop.org/software/systemd/man/systemd.service.html)
- [systemd.exec 手册](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)
- [journalctl 手册](https://www.freedesktop.org/software/systemd/man/journalctl.html)
