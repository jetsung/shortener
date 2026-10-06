# 部署说明

> **重要**：项目已采用**统一交付形态**（对齐 acmecast）——前端不再独立部署，
> 构建产物由统一镜像（`docker/Dockerfile`）在构建阶段内嵌，运行时由
> shortener-server 单进程直接托管（`SERVER__STATIC_DIR=/static`，未命中回退
> `index.html`）。标准部署方式请参阅
> [统一镜像部署指南](../deployment/DOCKER_AIO.md)。

## 目录

- [构建准备](#构建准备)
- [环境变量配置](#环境变量配置)

## 构建准备

以下步骤仅用于本地开发与质量检查；生产交付由统一镜像自动完成，无需手动构建。

### 系统要求

- Node.js 24（由仓库根目录 `mise.toml` 自动安装）或 >= 18.0.0
- pnpm 10.20.0（由 mise 自动安装）或 >= 10.0.0
- 至少 2GB 可用内存用于构建

> 安装 [mise](https://mise.jdx.dev) 后在仓库根目录执行 `mise install`，
> Node 与 pnpm 版本即自动就绪，无需手动管理。

### 构建步骤

1. **安装依赖**

```bash
pnpm install --frozen-lockfile
```

2. **类型检查**

```bash
pnpm type-check
```

3. **代码质量检查**

```bash
pnpm lint
```

4. **运行测试**

```bash
pnpm test
```

5. **生产构建（可选，镜像内已自动化）**

```bash
pnpm build
```

构建完成后，所有静态文件将生成在 `dist/` 目录中。

## 环境变量配置

### 开发环境 (.env.development)
