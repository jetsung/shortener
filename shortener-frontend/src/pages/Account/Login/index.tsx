import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, Form, Button, Typography } from '@douyinfe/semi-ui-19';
import { login } from '@/services/shortener/account';
import type { LoginForm } from '@/types';
import { Toast } from '@/utils/notification';

const { Title } = Typography;

const Login: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  // Handle OIDC callback: the server redirects back here with ?token=<jwt>&redirect=<hash path>.
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const token = params.get('token');
    if (token) {
      localStorage.setItem('token', token);
      // 登录后回跳：redirect 形如 `#/dashboard`，去掉 `#` 前缀交给 react-router
      const redirect = params.get('redirect');
      params.delete('token');
      params.delete('redirect');
      const newSearch = params.toString();
      const newHash = `${window.location.pathname}${newSearch ? `?${newSearch}` : ''}${window.location.hash}`;
      window.history.replaceState(null, '', newHash);
      Toast.success('登录成功');
      // 仅回跳到应用内路由（以 #/ 开头的 hash 路径），否则回 dashboard
      if (redirect && redirect.startsWith('#/')) {
        navigate(redirect.slice(1));
      } else {
        navigate('/dashboard');
      }
    }
  }, [navigate]);

  const handleOidcLogin = () => {
    // 后端固定跳转 /#/dashboard，无需再传 redirect 参数
    window.location.href = '/api/oidc/login';
  };

  const handleSubmit = async (values: LoginForm) => {
    setLoading(true);
    try {
      const response = await login({
        username: values.username,
        password: values.password,
      });

      // 处理不同可能的响应数据结构
      let token: string | undefined;
      let errorMessage: string | undefined;

      if (response && typeof response === 'object') {
        // 尝试不同的可能字段名
        const responseData = response as Record<string, unknown>;
        token = (responseData.token || responseData.access_token || responseData.accessToken) as
          | string
          | undefined;
        errorMessage = (responseData.errinfo || responseData.error || responseData.message) as
          | string
          | undefined;
      }

      if (token) {
        localStorage.setItem('token', token);

        // 验证token是否成功保存
        const _savedToken = localStorage.getItem('token');
        void _savedToken; // 标记为已使用

        Toast.success('登录成功');
        navigate('/dashboard');
      } else {
        Toast.error(errorMessage || '登录失败：未返回有效的token');
      }
    } catch (error: unknown) {
      console.error('登录错误:', error);
      const errorMessage = error instanceof Error ? error.message : '登录失败，请重试';
      Toast.error(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <style>{`
        .login-form-field .semi-form-field-main {
          margin-bottom: 24px;
        }
        .login-form-field .semi-form-field-error-message {
          position: absolute;
        }
      `}</style>
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'flex-start',
          minHeight: '100vh',
          paddingTop: '20vh',
          background: 'var(--semi-color-fill-0)',
          paddingLeft: '16px',
          paddingRight: '16px',
        }}
      >
        <Card
          style={{
            width: '100%',
            maxWidth: 400,
            padding: 24,
            boxShadow: 'var(--semi-shadow-elevated)',
          }}
        >
          <Title
            heading={2}
            style={{
              textAlign: 'center',
              marginBottom: 32,
              color: 'var(--semi-color-primary)',
            }}
          >
            Shortener
          </Title>

          <Form onSubmit={handleSubmit} style={{ marginBottom: 40 }}>
            <Form.Input
              field="username"
              label="用户名"
              placeholder="请输入用户名"
              rules={[{ required: true, message: '请输入用户名' }]}
              fieldClassName="login-form-field"
              autoComplete="username"
            />

            <Form.Input
              field="password"
              label="密码"
              type="password"
              placeholder="请输入密码"
              rules={[{ required: true, message: '请输入密码' }]}
              fieldClassName="login-form-field"
              autoComplete="current-password"
            />

            <Button type="primary" htmlType="submit" loading={loading} block size="large">
              登录
            </Button>

            <div
              style={{ textAlign: 'center', margin: '16px 0', color: 'var(--semi-color-text-2)' }}
            >
              或
            </div>

            <Button theme="light" block size="large" onClick={handleOidcLogin}>
              使用 OIDC 登录
            </Button>
          </Form>
        </Card>
      </div>
    </>
  );
};

export default Login;
