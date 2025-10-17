# 日志系统

此模块使用 `tracing` 和 `tracing-subscriber` crate 为 shortener 服务器提供全面的日志系统。

## 特性

- **可配置的日志级别**：支持 trace、debug、info、warn 和 error 级别
- **多种输出格式**：JSON、Pretty 和 Compact 格式
- **结构化日志**：包括时间戳、模块路径、线程信息等
- **环境变量支持**：可通过环境变量配置
- **灵活配置**：所有日志选项都可通过配置文件配置

## 配置

在 `config.toml` 中添加以下部分：

```toml
[logging]
# 日志级别：trace、debug、info、warn、error
level = "info"

# 日志格式：json、pretty、compact
format = "pretty"

# 在日志中包含时间戳
with_timestamp = true

# 在日志中包含目标（模块路径）
with_target = true

# 在日志中包含线程 ID
with_thread_ids = false

# 在日志中包含线程名称
with_thread_names = false

# 在日志中包含文件名
with_file = false

# 在日志中包含行号
with_line_number = false

# 使用 ANSI 颜色
with_ansi = true
```

## 使用

```rust
use shortener_server::logging;

// 初始化日志系统
logging::init(&config.logging)?;

// 使用日志宏
tracing::info!("服务器启动");
tracing::debug!("调试信息");
tracing::warn!("警告信息");
tracing::error!("错误信息");
```
