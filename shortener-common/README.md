# Shortener Common

服务器和 CLI 共享的类型、常量和工具。

## 内容

- **types.rs** - 通用数据结构（API 响应、分页、状态枚举）
- **errors.rs** - 错误类型和错误代码常量

## 使用

此库由 `shortener-server` 和 `shortener-cli` 内部使用。它提供：

- 跨包的一致类型定义
- 共享错误处理
- 通用常量和枚举
- 序列化/反序列化支持

## 开发

这是一个库 crate，不能直接运行。它会自动作为依赖包含在服务器和 CLI 包中。

运行测试：

```bash
cargo test -p shortener-common
```
