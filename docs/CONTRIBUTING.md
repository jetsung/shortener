# 贡献指南

感谢你考虑为 Shortener 项目做出贡献！

## 行为准则

请友善和尊重他人。我们致力于为每个人提供一个友好、安全和欢迎的环境。

## 如何贡献

### 报告 Bug

如果你发现了 bug，请在 [GitHub Issues](https://github.com/jetsung/shortener/issues) 创建一个问题，并包含：

- 清晰的标题和描述
- 重现步骤
- 预期行为和实际行为
- 你的环境信息（操作系统、Rust 版本等）
- 相关的日志或错误信息

### 建议新功能

我们欢迎新功能建议！请在 [GitHub Discussions](https://github.com/jetsung/shortener/discussions) 中讨论你的想法。

### 提交代码

1. **Fork 仓库**

   点击 GitHub 页面右上角的 "Fork" 按钮。

2. **克隆你的 Fork**

   ```bash
   git clone https://github.com/你的用户名/shortener.git
   cd shortener
   ```

3. **创建分支**

   ```bash
   git checkout -b feature/你的功能名称
   ```

4. **进行更改**

   - 遵循现有的代码风格
   - 为新功能编写测试
   - 更新相关文档
   - 确保所有测试通过

5. **提交更改**

   ```bash
   git add .
   git commit -m "添加：你的功能描述"
   ```

   提交信息格式：
   - `添加：` - 新功能
   - `修复：` - Bug 修复
   - `文档：` - 文档更新
   - `重构：` - 代码重构
   - `测试：` - 测试相关

6. **推送到你的 Fork**

   ```bash
   git push origin feature/你的功能名称
   ```

7. **创建 Pull Request**

   在 GitHub 上打开一个 Pull Request，描述你的更改。

## 开发设置

### 前提条件

- Rust 1.90 或更高版本
- Cargo（随 Rust 一起安装）
- Git

### 设置开发环境

```bash
# 克隆仓库
git clone https://github.com/jetsung/shortener.git
cd shortener

# 构建项目
cargo build

# 运行测试
cargo test

# 运行服务器
cargo run -p shortener-server
```

### 开发工具

安装推荐的开发工具：

```bash
# 代码格式化和检查
cargo install cargo-watch
cargo install cargo-audit

# 或使用 just
just install-tools
```

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 遵循 Rust 命名约定
- 为公共 API 编写文档注释

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定包的测试
cargo test -p shortener-server

# 运行测试并显示输出
cargo test -- --nocapture

# 运行基准测试
cargo bench
```

### 代码检查

在提交前运行：

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy -- -D warnings

# 运行测试
cargo test

# 或使用 just
just check
```

## 文档

### 更新文档

如果你的更改影响用户可见的行为，请更新相关文档：

- `README.md` - 项目概述
- `docs/` - 详细文档
- 代码注释 - 内联文档

### 构建文档

```bash
# 构建 Rust 文档
cargo doc --open

# 构建 MkDocs 文档
make docs-build

# 或使用 just
just docs-build
```

## Pull Request 指南

### 好的 PR 应该：

- 解决一个明确的问题或添加一个明确的功能
- 包含测试
- 更新相关文档
- 通过所有 CI 检查
- 有清晰的提交信息
- 保持较小的更改范围

### PR 流程

1. 确保你的分支是最新的
2. 所有测试通过
3. 代码已格式化和检查
4. 文档已更新
5. 提交 PR 并等待审查
6. 根据反馈进行修改
7. PR 被合并

## 发布流程

发布由维护者处理：

1. 更新版本号
2. 创建 Git 标签
3. 构建发布二进制文件
4. 发布到 GitHub Releases

## 获取帮助

如果你需要帮助：

- 📖 阅读 [文档](docs/)
- 💬 在 [Discussions](https://github.com/jetsung/shortener/discussions) 提问
- 🐛 在 [Issues](https://github.com/jetsung/shortener/issues) 报告问题
- 📧 联系维护者：i@jetsung.com

## 许可证

通过贡献，你同意你的贡献将在 Apache-2.0 许可证下授权。

## 致谢

感谢所有贡献者！你的贡献使这个项目变得更好。

---

**作者**: [Jetsung Chan](https://github.com/jetsung) <i@jetsung.com>
