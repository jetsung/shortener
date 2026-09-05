# 前端文档

Shortener Frontend 是一个基于 React 和 Semi Design 的现代化短链接管理界面。

## 📚 文档列表

- [组件文档](COMPONENTS.md) - 组件使用说明
- [部署指南](DEPLOYMENT.md) - 前端部署说明

## 🚀 快速开始

### 环境要求

- Node.js >= 18.0.0
- pnpm >= 10.0.0 (推荐) 或 npm

### 安装依赖

```bash
cd shortener-frontend

# 使用 pnpm (推荐)
pnpm install

# 或使用 npm
npm install
```

### 本地开发

```bash
# 启动开发服务器
pnpm dev

# 访问 http://localhost:8000
```

### 构建

```bash
# 生产构建
pnpm build

# 预览构建结果
pnpm preview
```

## ✨ 主要特性

- 🎨 基于 Semi Design 的现代化 UI
- 📱 响应式设计，支持移动端
- ⚡ Vite 快速构建和热重载
- 🔧 完整的 TypeScript 支持
- 🧪 全面的测试覆盖
- 🌐 国际化支持
- 🔄 短址列表页一键「刷新缓存」：清空缓存前缀旧键并从数据库重建，便于缓存与数据库不一致时快速恢复

## 🛠️ 技术栈

- **框架**: React 19.2.8
- **语言**: TypeScript 5.9.3
- **UI 库**: Semi Design 2.102.0
- **路由**: React Router DOM 7.18.3
- **构建工具**: Vite 7.3.6
- **测试**: Vitest 4.1.11

## 📖 详细文档

- [组件文档](COMPONENTS.md) - 了解如何使用各个组件
- [部署指南](DEPLOYMENT.md) - 了解如何部署到生产环境

## 🔧 配置

### 环境变量

创建 `.env.local` 文件：

```bash
# API 基础地址
VITE_API_BASE_URL=http://localhost:8080

# 应用标题
VITE_APP_TITLE=Shortener
```

### 主题定制

在 `config/theme.ts` 中自定义主题：

```typescript
export const semiTheme = {
  palette: {
    primary: '#1890ff',
    secondary: '#722ed1',
  },
};
```

## 🧪 测试

```bash
# 运行测试
pnpm test

# 生成覆盖率报告
pnpm test:coverage
```

## 📦 构建优化

项目已实现以下优化：

- 代码分割和懒加载
- 组件级别的性能优化
- 资源压缩和缓存
- Tree shaking

## 🌐 浏览器支持

- Chrome >= 60
- Firefox >= 55
- Safari >= 12
- Edge >= 79
