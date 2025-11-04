declare namespace API {
  type CurrentUser = {
    /** 用户名 */
    name?: string;
  };

  type deleteHistoriesParams = {
    /** id 列表，多个id用逗号分隔 */
    ids: string;
  };

  type deleteShortenParams = {
    /** id 列表，多个id用逗号分隔 */
    ids: string;
  };

  type Error = {
    /** 错误代码 */
    errcode?: number;
    /** 错误信息 */
    errinfo?: string;
  };

  type ErrorResponse = {
    /** 业务约定的错误码 */
    errcode: string;
    /** 业务上的错误信息 */
    errinfo?: string;
  };

  type getHistoriesParams = {
    /** 页码 */
    page?: number;
    /** 每页条数 */
    page_size?: number;
    /** 排序字段 */
    sort_by?: string;
    /** 排序方向 */
    order?: 'asc' | 'desc';
    /** 状态 */
    status?: 0 | 1 | 2;
  };

  type getShortensParams = {
    /** 页码 */
    page?: number;
    /** 每页条数 */
    page_size?: number;
    /** 排序字段 */
    sort_by?: string;
    /** 排序方向 */
    order?: 'asc' | 'desc';
    /** 状态 */
    status?: 0 | 1 | 2;
  };

  type HistoryResponse = Record<string, any>;

  type ItemList = {
    /** 数据列表 */
    data?: Record<string, any>[];
    meta?: PageMeta;
  };

  type LoginParams = {
    /** 用户名 */
    username?: string;
    /** 密码 */
    password?: string;
    /** 是否自动登录 */
    auto?: boolean;
  };

  type LoginResult = {
    /** 登录成功后返回的 token */
    token?: string;
    /** 业务约定的错误码 */
    errcode?: string;
    /** 业务上的错误信息 */
    errinfo?: string;
  };

  type PageMeta = {
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
  };

  type PageParams = {
    /** 页码 */
    page?: number;
    /** 每页数量 */
    page_size?: number;
    /** 排序字段 */
    sort_by?: string;
    /** 排序方式 */
    order?: 'asc' | 'desc';
  };

  type PageQuery = {
    /** 当前页码 */
    current?: number;
    /** 每页数量 */
    pageSize?: number;
  };

  type RespList = {
    /** 数据列表 */
    data?: Record<string, unknown>[];
    /** 请求是否成功 */
    success?: boolean;
    /** 总条目数 */
    total?: number;
  };

  type Shorten = {
    /** 原始长网址 */
    original_url: string;
    /** 短码 */
    short_code: string;
    /** 短链描述 */
    description?: string;
  };

  type ShortenResponse = {
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
  };

  type ShortenUpdate = {
    /** 原始长网址 */
    original_url: string;
    /** 短链描述 */
    description?: string;
  };

  type updateShortenParams = {
    /** 短码 */
    short_code: string;
  };
}
