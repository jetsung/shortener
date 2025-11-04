/**
 * 应用常量配置
 */

// API 相关常量
export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api';

// 应用信息
export const APP_CONFIG = {
  title: import.meta.env.VITE_APP_TITLE || 'Shortener',
  description: import.meta.env.VITE_APP_DESCRIPTION || 'URL 缩短服务',
  version: '1.0.0',
};

// 路由路径常量
export const ROUTES = {
  LOGIN: '/account/login',
  DASHBOARD: '/dashboard',
  SHORTENER: '/shortens',
  HISTORY: '/histories',
} as const;

// 本地存储键名
export const STORAGE_KEYS = {
  TOKEN: 'token',
  USER_INFO: 'userInfo',
  THEME: 'theme',
  LANGUAGE: 'language',
} as const;

// 主题相关常量
export const THEME_CONFIG = {
  LIGHT: 'light',
  DARK: 'dark',
} as const;

// 语言相关常量
export const LOCALE_CONFIG = {
  ZH_CN: 'zh-CN',
  EN_US: 'en-US',
} as const;
