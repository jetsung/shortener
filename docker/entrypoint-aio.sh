#!/bin/sh
# Shortener All-In-One 入口脚本
# 同时拉起 shortener-server（后端，setsid 独立会话）与 nginx（前台）
# 后端退出（崩溃）时由监控子 shell 终止 nginx，让容器整体退出；
# nginx 退出后主脚本结束。配合 docker restart 策略实现崩溃自愈。
set -e

# 使用 setsid 后台启动后端（独立会话，不受终端信号影响）
# 显式指定配置路径，不依赖 WORKDIR（镜像 WORKDIR 已设为 /app）
setsid /usr/local/bin/shortener-server -c /app/config.toml &
BACKEND_PID=$!

# 前台运行 nginx（对外提供静态资源与反向代理）
nginx -g 'daemon off;' &
NGINX_PID=$!

# 监控后端：轮询存活并回收。子 shell 无法 wait 父 shell 的后台任务，
# 故用 kill -0 检测（不依赖进程父子关系）
(
    set +e
    while kill -0 "$BACKEND_PID" 2>/dev/null; do
        sleep 2
    done
    echo "shortener-server exited unexpectedly, stopping nginx" >&2
    kill "$NGINX_PID" 2>/dev/null || true
) &
WATCHER_PID=$!

# 信号处理：终止后端与 nginx，等待清理完成
# shellcheck disable=SC2329
cleanup() {
    kill "$BACKEND_PID" "$NGINX_PID" "$WATCHER_PID" 2>/dev/null || true
    wait "$WATCHER_PID" 2>/dev/null || true
}
trap 'cleanup; exit 143' TERM INT
trap 'cleanup' EXIT

# 等待 nginx 退出（nginx 退出后整个容器终止）
wait "$NGINX_PID"
exit $?
