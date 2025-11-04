# 基准测试快速参考

## 运行基准测试

```bash
# 运行所有基准测试（需要几分钟）
cargo bench

# 运行特定基准测试套件
cargo bench --bench code_generation_bench
cargo bench --bench url_validation_bench
cargo bench --bench database_bench
cargo bench --bench cache_bench
cargo bench --bench service_bench

# 快速测试模式（用于 CI/CD）
cargo bench -- --test

# 运行特定基准测试
cargo bench -- "code_generation/6"

# 保存基线以供比较
cargo bench -- --save-baseline my-baseline

# 与基线比较
cargo bench -- --baseline my-baseline
```

## 基准测试套件

### 1. 代码生成（`code_generation_bench.rs`）
- 不同长度的代码生成（4-16 个字符）
- 不同的字符集
- 唯一性检查

**预期**：每个代码约 200-300 纳秒

### 2. URL 验证（`url_validation_bench.rs`）
- URL 格式验证
- 代码格式验证
- 字符集创建

**预期**：每次验证约 50-100 纳秒

### 3. 数据库操作（`database_bench.rs`）
- CRUD 操作
- 分页和过滤
- 批量操作

**预期**：大多数操作在亚毫秒级

## 性能目标

- 代码生成：< 500 纳秒
- URL 验证：< 200 纳秒
- 数据库查询：< 10 毫秒
- 缓存操作：< 1 毫秒

## 查看结果

基准测试结果保存在 `target/criterion/` 目录中。

使用浏览器打开 `target/criterion/report/index.html` 查看详细报告。
