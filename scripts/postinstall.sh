#!/usr/bin/env bash
set -e

# 创建必要的目录
mkdir -p /opt/shortener/config
mkdir -p /opt/shortener/data
mkdir -p /opt/shortener/logs

# 恢复或创建配置文件
if [[ -f /opt/shortener/config/config.toml.bak ]]; then
    mv /opt/shortener/config/config.toml.bak /opt/shortener/config/config.toml
elif [[ ! -f /opt/shortener/config/config.toml ]]; then
    cp /opt/shortener/config.toml.example /opt/shortener/config/config.toml
fi

# 设置权限
chown -R shortener:shortener /opt/shortener
chmod 640 /opt/shortener/config/config.toml

# 重载 systemd 并启用服务
systemctl daemon-reload
systemctl enable shortener-server.service

# 如果是首次安装，启动服务；如果是升级，重启服务
if [[ "$1" == "configure" ]] && [[ -z "$2" ]]; then
    # 首次安装
    echo "首次安装完成。请编辑 /opt/shortener/config/config.toml 配置文件，然后运行："
    echo "  sudo systemctl start shortener-server"
else
    # 升级
    systemctl restart shortener-server.service || true
fi

exit 0
