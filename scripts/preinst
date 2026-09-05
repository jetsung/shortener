#!/usr/bin/env bash
set -e

# 创建系统用户和组
if ! getent group shortener >/dev/null; then
    groupadd --system shortener
fi

if ! getent passwd shortener >/dev/null; then
    useradd --system --gid shortener --home-dir /opt/shortener --no-create-home --shell /usr/sbin/nologin shortener
fi

# 备份现有配置
if [[ -f /opt/shortener/config/config.toml ]]; then
    cp /opt/shortener/config/config.toml /opt/shortener/config/config.toml.bak
fi

exit 0
