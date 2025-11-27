import React from 'react';
import { createRoot } from 'react-dom/client';
import { HashRouter } from 'react-router-dom';
import { LocaleProvider } from '@douyinfe/semi-ui';
import zh_CN from '@douyinfe/semi-ui/lib/es/locale/source/zh_CN';
import App from './App';
import { applyThemeTokens } from '../config/theme';
import { initWebVitals, PerformanceMonitor } from './utils/performance';

// 全局样式
import './global.css';

// 应用主题令牌
applyThemeTokens();

// 初始化性能监控
initWebVitals();

// 开始应用启动性能测量
const performanceMonitor = PerformanceMonitor.getInstance();
performanceMonitor.startMeasure('app-startup');

const container = document.getElementById('root');
if (!container) throw new Error('Failed to find the root element');

const root = createRoot(container);

root.render(
  <React.StrictMode>
    <HashRouter>
      <LocaleProvider locale={zh_CN}>
        <App />
      </LocaleProvider>
    </HashRouter>
  </React.StrictMode>,
);

// 结束应用启动性能测量
performanceMonitor.endMeasure('app-startup');
