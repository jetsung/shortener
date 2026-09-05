/* eslint-disable */
import request from '../request';

/** 刷新缓存 清空缓存前缀下所有旧键，并从数据库重新加载全部短链 POST /cache/refresh */
export async function refreshCache(options?: { [key: string]: any }) {
  return request<API.CacheRefreshResponse>('/cache/refresh', {
    method: 'POST',
    ...(options || {}),
  });
}
