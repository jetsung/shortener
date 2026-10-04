import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';
import process from 'node:process';

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');

  return {
    plugins: [react()],
    define: {
      'process.env.NODE_ENV': JSON.stringify(env.NODE_ENV || mode || 'development'),
      global: 'globalThis',
    },
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
    server: {
      // 端口以 VITE_DEV_PORT 为准（缺省 8000）；strictPort 使端口被占用时
      // 显式报错退出，而非静默漂移到 8001 等端口
      port: Number(env.VITE_DEV_PORT) || 8000,
      strictPort: true,
      proxy: {
        '/api': {
          // 指向 127.0.0.1 而非 localhost：Node ≥17 下 localhost 可能解析为
          // IPv6 ::1，后端仅监听 IPv4，会导致代理连接被拒（对齐 acmecast）
          target: 'http://127.0.0.1:8080',
          changeOrigin: false,
        },
      },
    },
    build: {
      outDir: 'dist',
      sourcemap: true,
      rolldownOptions: {
        // 关闭第三方库噪音告警：
        // - eval：semi-foundation 的传递依赖 lottie-web 内部使用了 direct eval，
        //   但该代码已被 tree-shake，不会进入产物
        // - pluginTimings：仅是构建性能剖析提示，非性能问题
        checks: {
          eval: false,
          pluginTimings: false,
        },
        output: {
          manualChunks: (id) => {
            // Semi Design 组件单独打包
            if (id.includes('@douyinfe/semi-ui-19')) {
              return 'semi-ui';
            }
            if (id.includes('@douyinfe/semi-icons')) {
              return 'semi-icons';
            }

            // React 相关库单独打包
            if (id.includes('react') || id.includes('react-dom')) {
              return 'react-vendor';
            }

            // 路由相关库单独打包
            if (id.includes('react-router')) {
              return 'router';
            }

            // 工具库单独打包
            if (id.includes('axios') || id.includes('dayjs') || id.includes('classnames')) {
              return 'utils';
            }

            // node_modules 中的其他库
            if (id.includes('node_modules')) {
              return 'vendor';
            }
          },
          // 优化文件名和缓存
          chunkFileNames: 'assets/js/[name]-[hash].js',
          entryFileNames: 'assets/js/[name]-[hash].js',
          assetFileNames: 'assets/[ext]/[name]-[hash].[ext]',
        },
      },
      // 启用压缩和优化
      minify: 'terser',
      terserOptions: {
        compress: {
          drop_console: true,
          drop_debugger: true,
        },
      },
      // 设置 chunk 大小警告阈值
      // semi-ui 全量组件打包后约 1.1MB（gzip 约 280KB），属合理体积
      chunkSizeWarningLimit: 1200,
    },
    css: {
      preprocessorOptions: {
        less: {
          javascriptEnabled: true,
        },
      },
    },
    optimizeDeps: {
      include: [
        '@douyinfe/semi-ui-19',
        '@douyinfe/semi-icons',
        'react',
        'react-dom',
        'react-router-dom',
        'axios',
        'dayjs',
        'classnames',
      ],
      // 强制预构建这些依赖以提高开发服务器性能
      force: true,
    },
  };
});
