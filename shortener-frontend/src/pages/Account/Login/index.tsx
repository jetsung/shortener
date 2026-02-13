import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, Form, Button, Typography } from '@douyinfe/semi-ui-19';
import { login } from '@/services/shortener/account';
import type { LoginForm } from '@/types';
import { Toast } from '@/utils/notification';

const { Title } = Typography;

const Login: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

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
        const responseData = response as any;
        token = responseData.token || responseData.access_token || responseData.accessToken;
        errorMessage = responseData.errinfo || responseData.error || responseData.message;
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
          </Form>
        </Card>
      </div>
    </>
  );
};

export default Login;
