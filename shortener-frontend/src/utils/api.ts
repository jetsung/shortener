/**
 * API 工具函数
 */

import { Toast } from '@douyinfe/semi-ui-19';

/**
 * 处理 API 响应
 */
export const handleApiResponse = <T>(response: any): T => {
  if (response && typeof response === 'object') {
    // 如果有 success 字段且为 false，抛出错误
    if ('success' in response && !response.success) {
      throw new Error(response.message || response.errinfo || '请求失败');
    }

    // 如果有 errcode 字段且不为 0，抛出错误
    if ('errcode' in response && response.errcode !== 0) {
      throw new Error(response.errinfo || '请求失败');
    }

    // 返回数据部分
    return response.data || response;
  }

  return response;
};

/**
 * 处理 API 错误
 */
export const handleApiError = (error: any, defaultMessage = '操作失败') => {
  console.error('API Error:', error);

  let message = defaultMessage;

  if (error?.response?.data?.message) {
    message = error.response.data.message;
  } else if (error?.response?.data?.errinfo) {
    message = error.response.data.errinfo;
  } else if (error?.message) {
    message = error.message;
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
  window.location.href = '/account/login';
};
