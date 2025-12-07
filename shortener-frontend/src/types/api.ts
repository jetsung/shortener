// API 类型定义
export interface CurrentUser {
  /** 用户名 */
  name?: string;
}

export interface DeleteHistoriesParams {
  /** id 列表，多个id用逗号分隔 */
  ids: string;
}

export interface DeleteShortenParams {
  /** id 列表，多个id用逗号分隔 */
  ids: string;
}

export interface Error {
  /** 错误代码 */
  errcode?: number;
  /** 错误信息 */
  errinfo?: string;
}

export interface ErrorResponse {
  /** 业务约定的错误码 */
  errcode: string;
  /** 业务上的错误信息 */
  errinfo?: string;
}

export interface GetHistoriesParams {
  /** 页码 */
  page?: number;
  /** 每页条数 */
  per_page?: number;
  /** 排序字段 */
  sort_by?: string;
  /** 排序方向 */
  order?: 'asc' | 'desc';
  /** 短码搜索 */
  short_code?: string;
  /** IP地址搜索 */
  ip_address?: string;
}

export interface GetShortensParams {
  /** 页码 */
  page?: number;
  /** 每页条数 */
  per_page?: number;
  /** 排序字段 */
  sort_by?: string;
  /** 排序方向 */
  order?: 'asc' | 'desc';
  /** 状态 */
  status?: 0 | 1 | 2;
  /** 短码搜索 */
  short_code?: string;
  /** 原始URL搜索（模糊匹配） */
  original_url?: string;
}

export interface HistoryResponse {
  /** 访问记录 ID */
  id: number;
  /** 短链接 ID */
  url_id: number;
  /** 短码 */
  short_code: string;
  /** 访问者 IP */
  ip_address: string;
  /** 浏览器 UA */
  user_agent: string;
  /** 来源页面 */
  referer?: string | null;
  /** 国家 */
  country?: string;
  /** 地区 */
  region?: string;
  /** 省份 */
  province?: string;
  /** 城市 */
  city?: string;
  /** ISP */
  isp?: string;
  /** 设备类型 */
  device_type?: 'pc' | 'mobile' | 'tablet';
  /** 操作系统 */
  os?: 'Windows' | 'MacOS' | 'Linux' | 'Android' | 'iOS';
  /** 浏览器 */
  browser?: 'Chrome' | 'Firefox' | 'Safari' | 'Edge' | 'IE';
  /** 访问时间 (ISO 8601 格式) */
  accessed_at: string;
  /** 记录创建时间 (ISO 8601 格式) */
  created_at: string;
}

export interface ItemList {
  /** 数据列表 */
  data?: Record<string, any>[];
  meta?: PageMeta;
}

export interface LoginParams {
  /** 用户名 */
  username?: string;
  /** 密码 */
  password?: string;
  /** 是否自动登录 */
  auto?: boolean;
}

export interface LoginResult {
  /** 登录成功后返回的 token */
  token?: string;
  /** 业务约定的错误码 */
  errcode?: string;
  /** 业务上的错误信息 */
  errinfo?: string;
}

export interface PageMeta {
  /** 当前页码 */
  page: number;
  /** 每页数量 */
  per_page: number;
  /** 当前页条目数 */
  count: number;
  /** 总条目数 */
  total: number;
  /** 总页数 */
  total_pages: number;
}

export interface PageParams {
  /** 页码 */
  page?: number;
  /** 每页数量 */
  per_page?: number;
  /** 排序字段 */
  sort_by?: string;
  /** 排序方式 */
  order?: 'asc' | 'desc';
}

export interface PageQuery {
  /** 当前页码 */
  current?: number;
  /** 每页数量 */
  pageSize?: number;
}

export interface RespList {
  /** 数据列表 */
  data?: Record<string, any>[];
  /** 请求是否成功 */
  success?: boolean;
  /** 总条目数 */
  total?: number;
}

export interface Shorten {
  /** 原始长网址 */
  original_url: string;
  /** 短码 */
  short_code: string;
  /** 短链描述 */
  description?: string;
}

export interface ShortenResponse {
  /** ID */
  id?: number;
  /** 短码 */
  short_code?: string;
  /** 短网址 */
  short_url?: string;
  /** 原始长网址 */
  original_url?: string;
  /** 短链描述 */
  description?: string;
  /** 状态：0=启用, 1=禁用, 2=未知 */
  status?: 0 | 1 | 2;
  /** 创建时间 (ISO 8601 格式) */
  created_at?: string;
  /** 更新时间 (ISO 8601 格式) */
  updated_at?: string;
}

export interface ShortenUpdate {
  /** 原始长网址 */
  original_url: string;
  /** 短链描述 */
  description?: string;
}

export interface UpdateShortenParams {
  /** 短码 */
  short_code: string;
}
