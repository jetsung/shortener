import axios, { AxiosResponse } from 'axios';
import { Toast } from '@/utils/notification';
import { redirectToLogin } from '@/utils/api';

// 创建 axios 实例
const request = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器
request.interceptors.request.use(
  (config) => {
    // 添加 token 等认证信息
    const token = localStorage.getItem('token');

    if (token && token !== '' && config.headers) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    return config;
  },
  (error) => {
    return Promise.reject(error);
  },
);

// 响应拦截器
request.interceptors.response.use(
  (response: AxiosResponse) => {
    const { data } = response;

    // 直接返回响应数据，让业务层处理
    return data;
  },
  (error) => {
    // 处理 HTTP 错误状态码
    if (error.response) {
      const { status, data } = error.response;

      switch (status) {
        case 401: {
          Toast.error('未授权，请重新登录');
          // 如果当前不在登录页面，才进行重定向
          const currentPath = window.location.hash || window.location.pathname;
          if (!currentPath.includes('/account/login')) {
            redirectToLogin();
          }
          break;
        }
        case 403:
          Toast.error('权限不足');
          break;
        case 404:
          Toast.error('请求的资源不存在');
          break;
        case 500:
          Toast.error('服务器内部错误');
          break;
        default:
          Toast.error(data?.message || `请求失败 (${status})`);
      }
    } else if (error.request) {
      Toast.error('网络错误，请检查网络连接');
    } else {
      Toast.error('请求配置错误');
    }

    return Promise.reject(error);
  },
);

export default request;
