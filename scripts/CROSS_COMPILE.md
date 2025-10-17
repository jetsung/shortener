# 交叉编译指南

本指南说明如何为不同平台和架构交叉编译 Shortener 项目。

## 前提条件

### 安装 Cross

[Cross](https://github.com/cross-rs/cross) 是一个使用 Docker 交叉编译 Rust 项目的工具：

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### 安装 Docker

Cross 需要安装并运行 Docker：

- **Linux**：遵循 [Docker 安装指南](https://docs.docker.com/engine/install/)
- **macOS**：安装 [Docker Desktop](https://www.docker.com/products/docker-desktop)
- **Windows**：安装 [Docker Desktop](https://www.docker.com/products/docker-desktop)

## 支持的目标平台

### Linux

- `x86_64-unknown-linux-gnu` - Linux x86_64 with glibc（最常见）
- `x86_64-unknown-linux-musl` - Linux x86_64 with musl（静态链接）
- `aarch64-unknown-linux-gnu` - Linux ARM64 with glibc
- `aarch64-unknown-linux-musl` - Linux ARM64 with musl（静态链接）
- `armv7-unknown-linux-gnueabihf` - Linux ARMv7（树莓派等）

### Windows

- `x86_64-pc-windows-gnu` - Windows x86_64

### macOS

- `x86_64-apple-darwin` - macOS x86_64（Intel）
- `aarch64-apple-darwin` - macOS ARM64（Apple Silicon）

**注意**：macOS 目标需要在 macOS 上构建或使用 osxcross。

## 快速开始

### 构建所有目标

```bash
# 为所有目标构建服务器
./scripts/build-cross.sh --server

# 为所有目标构建 CLI
./scripts/build-cross.sh --cli

# 构建所有
./scripts/build-cross.sh --all
```

### 构建特定目标

```bash
# 为 Linux x86_64（musl）构建服务器
./scripts/build-cross.sh -t x86_64-unknown-linux-musl --server

# 为 ARM64 构建 CLI
./scripts/build-cross.sh -t aarch64-unknown-linux-gnu --cli
```

### 列出可用目标

```bash
./scripts/build-cross.sh --list
```

## 手动交叉编译

### 使用 Cross

```bash
# 为 Linux x86_64（musl）构建
cross build --release --target x86_64-unknown-linux-musl -p shortener-server

# 为 ARM64 构建
cross build --release --target aarch64-unknown-linux-gnu -p shortener-server

# 为 Windows 构建
cross build --release --target x86_64-pc-windows-gnu -p shortener-server
```

### 使用 Cargo（原生）

对于主机系统支持的目标：

```bash
# 添加目标
rustup target add x86_64-unknown-linux-musl

# 构建
cargo build --release --target x86_64-unknown-linux-musl -p shortener-server
```

## 输出位置

二进制文件位于：

```
target/<target>/release/
├── shortener-server
└── shortener-cli
```

打包的发布版本位于：

```
target/release-builds/<target>/
├── shortener-server-<version>-<target>.tar.gz
├── shortener-server-<version>-<target>.sha256
├── shortener-cli-<version>-<target>.tar.gz
└── shortener-cli-<version>-<target>.sha256
```

## 平台特定说明

### Linux（musl）

Musl 目标生成可在任何 Linux 发行版上运行的静态链接二进制文件：

```bash
cross build --release --target x86_64-unknown-linux-musl -p shortener-server
```

优点：
- 无运行时依赖
- 可在任何 Linux 发行版上运行
- 更小的二进制大小

### ARM 平台

对于树莓派和其他 ARM 设备：

```bash
# ARMv7（树莓派 2/3/4 32 位模式）
cross build --release --target armv7-unknown-linux-gnueabihf -p shortener-server

# ARM64（树莓派 3/4 64 位模式）
cross build --release --target aarch64-unknown-linux-gnu -p shortener-server
```

### Windows

从 Linux 交叉编译到 Windows：

```bash
cross build --release --target x86_64-pc-windows-gnu -p shortener-server
```

输出将是 `shortener-server.exe`。

### macOS

#### 在 macOS 上（原生）

```bash
# Intel Mac
cargo build --release --target x86_64-apple-darwin -p shortener-server

# Apple Silicon
cargo build --release --target aarch64-apple-darwin -p shortener-server

# 通用二进制（两种架构）
cargo build --release --target x86_64-apple-darwin -p shortener-server
cargo build --release --target aarch64-apple-darwin -p shortener-server
lipo -create \
  target/x86_64-apple-darwin/release/shortener-server \
  target/aarch64-apple-darwin/release/shortener-server \
  -output shortener-server-universal
```

## 故障排除

### 找不到 Cross

```bash
# 安装 cross
cargo install cross --git https://github.com/cross-rs/cross

# 验证安装
cross --version
```

### Docker 未运行

```bash
# 启动 Docker
sudo systemctl start docker  # Linux
# 或在 macOS/Windows 上打开 Docker Desktop

# 验证 Docker
docker ps
```

### 权限被拒绝（Linux）

```bash
# 将用户添加到 docker 组
sudo usermod -aG docker $USER

# 注销并重新登录，或运行：
newgrp docker
```

### 构建失败

```bash
# 清理构建
cargo clean

# 更新 cross
cargo install cross --git https://github.com/cross-rs/cross --force

# 拉取最新的 Docker 镜像
docker pull ghcr.io/cross-rs/x86_64-unknown-linux-gnu:latest
```

## 优化

### 减小二进制大小

在 `Cargo.toml` 中添加：

```toml
[profile.release]
opt-level = "z"     # 优化大小
lto = true          # 启用链接时优化
codegen-units = 1   # 更好的优化
strip = true        # 剥离符号
panic = "abort"     # 更小的 panic 处理器
```

### 更快的构建

```bash
# 使用 sccache 进行缓存
cargo install sccache
export RUSTC_WRAPPER=sccache

# 并行构建
export CARGO_BUILD_JOBS=8
```

## 最佳实践

1. **在目标平台上测试** - 始终在实际目标平台上测试二进制文件
2. **Linux 使用 musl** - 生成可移植的静态二进制文件
3. **剥离二进制文件** - 删除调试符号以减小大小
4. **验证校验和** - 始终提供并验证 SHA256 校验和
5. **记录依赖项** - 列出任何运行时依赖项
6. **一致的版本控制** - 使用语义化版本
7. **自动化构建** - 使用 CI/CD 进行一致的构建

## 参考

- [Cross 文档](https://github.com/cross-rs/cross)
- [Rust 平台支持](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Cargo 手册 - 构建脚本](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- [rustup 文档](https://rust-lang.github.io/rustup/)
