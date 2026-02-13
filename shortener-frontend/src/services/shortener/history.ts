/* eslint-disable */
import request from '../request';

/** 获取所有日志信息 获取所有日志信息 GET /histories */
export async function getHistories(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getHistoriesParams,
  options?: { [key: string]: any },
) {
  return request<{ data?: API.HistoryResponse[]; meta?: API.PageMeta }>('/histories', {
    method: 'GET',
    params: {
      // page has a default value: 1
      page: '1',
      // per_page has a default value: 10
      per_page: '10',
      // sort_by has a default value: accessed_at
      sort_by: 'accessed_at',
      // order has a default value: desc
      order: 'desc',
      ...params,
    },
    ...(options || {}),
  });
}

/** 删除日志列表 删除日志列表 返回值: 未知错误 POST /histories/batch-delete */
export async function deleteHistories(
  body: API.BatchDeleteRequest,
  options?: { [key: string]: any },
) {
  return request<API.Error>('/histories/batch-delete', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}
