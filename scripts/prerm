#!/usr/bin/env bash
set -e

# 停止并禁用服务
if systemctl is-active --quiet shortener-server.service; then
    systemctl stop shortener-server.service
fi

if systemctl is-enabled --quiet shortener-server.service; then
    systemctl disable shortener-server.service
fi

exit 0
