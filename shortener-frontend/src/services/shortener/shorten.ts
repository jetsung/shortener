/* eslint-disable */
import request from '../request';

/** 获取所有短址信息 获取所有短址信息 GET /shortens */
export async function getShortens(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.getShortensParams,
  options?: { [key: string]: any },
) {
  return request<{ data?: API.ShortenResponse[]; meta?: API.PageMeta }>('/shortens', {
    method: 'GET',
    params: {
      // page has a default value: 1
      page: '1',
      // per_page has a default value: 10
      per_page: '10',
      // sort_by has a default value: created_at
      sort_by: 'created_at',
      // order has a default value: desc
      order: 'desc',
      ...params,
    },
    ...(options || {}),
  });
}

/** 添加短网址 添加一个新的短网址 返回值: 未知错误 POST /shortens */
export async function addShorten(body: API.Shorten, options?: { [key: string]: any }) {
  return request<API.Error>('/shortens', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** 删除短网址列表 删除短网址列表 返回值: 未知错误 POST /shortens/batch-delete */
export async function deleteShorten(
  body: API.BatchDeleteRequest,
  options?: { [key: string]: any },
) {
  return request<API.Error>('/shortens/batch-delete', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** 更新短网址 更新一个短网址 返回值: 未知错误 PUT /shortens/${param0} */
export async function updateShorten(
  // 叠加生成的Param类型 (非body参数swagger默认没有生成对象)
  params: API.updateShortenParams,
  body: API.ShortenUpdate,
  options?: { [key: string]: any },
) {
  const { short_code: param0, ...queryParams } = params;
  return request<API.Error>(`/shortens/${param0}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    params: { ...queryParams },
    data: body,
    ...(options || {}),
  });
}
