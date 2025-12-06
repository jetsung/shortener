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
      console.log('开始登录，用户名:', values.username);

      const response = await login({
        username: values.username,
        password: values.password,
      });

      console.log('登录API完整响应:', response);
      console.log('响应数据类型:', typeof response);
      console.log('响应数据结构:', JSON.stringify(response, null, 2));

      // 处理不同可能的响应数据结构
      let token: string | undefined;
      let errorMessage: string | undefined;

      if (response && typeof response === 'object') {
        // 尝试不同的可能字段名
        const responseData = response as any;
        token = responseData.token || responseData.access_token || responseData.accessToken;
        errorMessage = responseData.errinfo || responseData.error || responseData.message;

        console.log('提取的token:', token);
        console.log('提取的错误信息:', errorMessage);
      }

      if (token) {
        console.log('保存token到localStorage:', token);
        localStorage.setItem('token', token);

        // 验证token是否成功保存
        const savedToken = localStorage.getItem('token');
        console.log('验证localStorage中的token:', savedToken);

        Toast.success('登录成功');
        navigate('/dashboard');
      } else {
        console.log('未找到token，登录失败');
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
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        minHeight: '100vh',
        background: 'var(--semi-color-fill-0)',
      }}
    >
      <Card
        style={{
          width: 400,
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

        <Form onSubmit={handleSubmit}>
          <Form.Input
            field="username"
            label="用户名"
            placeholder="请输入用户名"
            rules={[{ required: true, message: '请输入用户名' }]}
            style={{ marginBottom: 16 }}
            autoComplete="username"
          />

          <Form.Input
            field="password"
            label="密码"
            type="password"
            placeholder="请输入密码"
            rules={[{ required: true, message: '请输入密码' }]}
            style={{ marginBottom: 24 }}
            autoComplete="current-password"
          />

          <Button type="primary" htmlType="submit" loading={loading} block size="large">
            登录
          </Button>
        </Form>
      </Card>
    </div>
  );
};

export default Login;
