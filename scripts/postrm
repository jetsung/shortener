#!/usr/bin/env bash
set -e

# 重载 systemd
systemctl daemon-reload

# 如果是完全卸载（purge），清理所有文件
if [[ "$1" == "purge" ]]; then
    rm -rf /opt/shortener

    # 删除用户和组
    if getent passwd shortener >/dev/null; then
        userdel shortener
    fi

    if getent group shortener >/dev/null; then
        groupdel shortener
    fi

    echo "已完全卸载 shortener-server 及其所有数据"
fi

exit 0
