/* eslint-disable */
import request from '../request';

/** 登录接口 POST /account/login */
export async function login(body: API.LoginParams, options?: { [key: string]: any }) {
  return request<API.LoginResult>('/account/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    data: body,
    ...(options || {}),
  });
}

/** 退出登录接口 POST /account/logout */
export async function logout(options?: { [key: string]: any }) {
  return request<any>('/account/logout', {
    method: 'POST',
    ...(options || {}),
  });
}

/** 获取当前的用户 GET /users/current */
export async function currentUser(options?: { [key: string]: any }) {
  return request<API.CurrentUser>('/users/current', {
    method: 'GET',
    ...(options || {}),
  });
}
