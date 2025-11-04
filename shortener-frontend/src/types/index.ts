/**
 * 全局类型定义
 */

// 用户相关类型
export interface User {
  id: string;
  username: string;
  email?: string;
  avatar?: string;
  createdAt: string;
  updatedAt: string;
}

// 短链接相关类型
export interface ShortUrl {
  id: string;
  originalUrl: string;
  shortCode: string;
  shortUrl: string;
  title?: string;
  description?: string;
  clickCount: number;
  isActive: boolean;
  expiresAt?: string;
  createdAt: string;
  updatedAt: string;
  userId: string;
}

// 访问历史相关类型
export interface ClickHistory {
  id: string;
  shortUrlId: string;
  shortUrl: ShortUrl;
  ipAddress: string;
  userAgent: string;
  referer?: string;
  country?: string;
  city?: string;
  clickedAt: string;
}

// API 响应类型
export interface ApiResponse<T = any> {
  success: boolean;
  data: T;
  message?: string;
  code?: number;
}

// 分页相关类型
export interface PaginationParams {
  page: number;
  pageSize: number;
  total?: number;
}

export interface PaginatedResponse<T> {
  list: T[];
  pagination: {
    current: number;
    pageSize: number;
    total: number;
    totalPages: number;
  };
}

// 表单相关类型
export interface LoginForm {
  username: string;
  password: string;
}

export interface CreateShortUrlForm {
  originalUrl: string;
  customCode?: string;
  title?: string;
  description?: string;
  expiresAt?: string;
}

import React from 'react';

// 路由相关类型
export interface RouteItem {
  path: string;
  name: string;
  icon?: React.ReactNode;
  component?: React.ComponentType;
  children?: RouteItem[];
}

// 主题相关类型
export type ThemeMode = 'light' | 'dark';

// 语言相关类型
export type Locale = 'zh-CN' | 'en-US';

// 表格列配置类型
export interface TableColumn<T = any> {
  title: string;
  dataIndex: keyof T;
  key: string;
  width?: number;
  align?: 'left' | 'center' | 'right';
  render?: (value: any, record: T, index: number) => React.ReactNode;
  sorter?: boolean;
  filters?: Array<{ text: string; value: any }>;
}

// 统计数据类型
export interface DashboardStats {
  totalUrls: number;
  totalClicks: number;
  todayClicks: number;
  activeUrls: number;
}
