# 部署说明

> **重要**：项目已采用**统一交付形态**——前端不再独立部署，构建产物由统一镜像
> （`docker/Dockerfile`）在构建阶段内嵌，运行时由 nginx 托管（后端
> `SERVER__STATIC_DIR` 兜底，含 SPA 回退）。标准部署方式请参阅
> [All-In-One Docker 部署](../deployment/DOCKER_AIO.md)。

## 目录

- [构建准备](#构建准备)
- [环境变量配置](#环境变量配置)

## 构建准备

以下步骤仅用于本地开发与质量检查；生产交付由统一镜像自动完成，无需手动构建。

### 系统要求

- Node.js >= 18.0.0
- pnpm >= 10.0.0 (推荐) 或 npm >= 9.0.0
- 至少 2GB 可用内存用于构建

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
