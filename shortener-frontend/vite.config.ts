import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import { fileURLToPath, URL } from 'node:url'
import process from 'node:process'


// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');

  return {
    plugins: [react()],
    define: {
      'process.env.NODE_ENV': JSON.stringify(env.NODE_ENV || mode || 'development'),
      'global': 'globalThis',
    },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 8000,
    proxy: {
      '/api': {
        // target: 'https://dwz.asfd.cn',
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false, // 修改为 false，因为目标是 http
        rewrite: (path) => path.replace(/^\/api/, '/api'),
        configure: (proxy, _options) => {
          proxy.on('error', (err, _req, _res) => {
            console.log('proxy error', err);
          });
          proxy.on('proxyReq', (proxyReq, req, _res) => {
            console.log('Sending Request to the Target:', req.method, req.url);
            // 确保正确设置请求头
            proxyReq.setHeader('Accept', 'application/json');
            proxyReq.setHeader('Content-Type', 'application/json');
          });
          proxy.on('proxyRes', (proxyRes, req, _res) => {
            console.log('Received Response from the Target:', proxyRes.statusCode, req.url);
            console.log('Response Headers:', proxyRes.headers);
          });
        },
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Semi Design 组件单独打包
          if (id.includes('@douyinfe/semi-ui')) {
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
    chunkSizeWarningLimit: 1000,
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
      '@douyinfe/semi-ui',
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
