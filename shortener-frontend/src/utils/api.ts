/**
 * API 工具函数
 */

import { Toast } from '@douyinfe/semi-ui-19';

/**
 * 处理 API 响应
 */
export const handleApiResponse = <T>(response: unknown): T => {
  if (response && typeof response === 'object') {
    const res = response as Record<string, unknown>;

    // 如果有 success 字段且为 false，抛出错误
    if ('success' in res && !res.success) {
      throw new Error(String(res.message || res.errinfo || '请求失败'));
    }

    // 如果有 errcode 字段且不为 0，抛出错误
    if ('errcode' in res && res.errcode !== 0) {
      throw new Error(String(res.errinfo || '请求失败'));
    }

    // 返回数据部分
    return (res.data || res) as T;
  }

  return response as T;
};

/**
 * 处理 API 错误
 */
export const handleApiError = (error: unknown, defaultMessage = '操作失败') => {
  console.error('API Error:', error);

  const err = error as {
    response?: { data?: { message?: string; errinfo?: string } };
    message?: string;
  };

  let message = defaultMessage;

  if (err?.response?.data?.message) {
    message = err.response.data.message;
  } else if (err?.response?.data?.errinfo) {
    message = err.response.data.errinfo;
  } else if (err?.message) {
    message = err.message;
  }

  Toast.error(message);
  return message;
};

/**
 * 检查用户是否已登录
 */
export const isAuthenticated = (): boolean => {
  return !!localStorage.getItem('token');
};

/**
 * 获取当前用户 token
 */
export const getToken = (): string | null => {
  return localStorage.getItem('token');
};

/**
 * 清除用户认证信息
 */
export const clearAuth = (): void => {
  localStorage.removeItem('token');
};

/**
 * 跳转到登录页
 */
export const redirectToLogin = (): void => {
  clearAuth();
  window.location.href = '/#/account/login';
};
