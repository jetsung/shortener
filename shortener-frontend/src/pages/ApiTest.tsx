import React, { useState } from 'react';
import { Button, Card, Typography, Space, Toast } from '@douyinfe/semi-ui';
import { login } from '@/services/shortener/account';

const { Title, Text } = Typography;

const ApiTest: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  const testLogin = async () => {
    setLoading(true);
    setResponse(null);
    setError(null);

    try {
      console.log('开始测试登录API...');

      const result = await login({
        username: 'admin', // 使用更常见的测试用户名
        password: 'admin123',
      });

      console.log('登录API响应:', result);
      console.log('响应数据类型:', typeof result);

      // 检查可能的token字段
      const resultData = result as any;
      const possibleTokens = {
        token: resultData?.token,
        access_token: resultData?.access_token,
        accessToken: resultData?.accessToken,
        data_token: resultData?.data?.token,
      };

      console.log('可能的token字段:', possibleTokens);

      setResponse({
        originalResponse: result,
        possibleTokens,
        responseType: typeof result,
        isObject: result && typeof result === 'object',
        keys: result && typeof result === 'object' ? Object.keys(result) : [],
      });
      Toast.success('API调用成功');
    } catch (err: any) {
      console.error('登录API错误:', err);
      setError(err.message || '未知错误');
      Toast.error('API调用失败');
    } finally {
      setLoading(false);
    }
  };

  const testDirectFetch = async () => {
    setLoading(true);
    setResponse(null);
    setError(null);

    try {
      console.log('开始测试直接fetch...');

      const response = await fetch('/api/account/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username: 'admin',
          password: 'admin123',
        }),
      });

      console.log('Fetch响应状态:', response.status);
      console.log('Fetch响应头:', response.headers);

      const data = await response.text();
      console.log('Fetch响应数据:', data);

      setResponse({
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        data: data,
      });

      Toast.success('Fetch调用成功');
    } catch (err: any) {
      console.error('Fetch错误:', err);
      setError(err.message || '未知错误');
      Toast.error('Fetch调用失败');
    } finally {
      setLoading(false);
    }
  };

  const testCurrentUser = async () => {
    setLoading(true);
    setResponse(null);
    setError(null);

    try {
      console.log('开始测试获取当前用户API...');

      const token = localStorage.getItem('token');
      console.log('当前localStorage中的token:', token);

      const response = await fetch('/api/users/current', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
      });

      console.log('获取用户信息响应状态:', response.status);
      console.log('获取用户信息响应头:', response.headers);

      const data = await response.text();
      console.log('获取用户信息响应数据:', data);

      setResponse({
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        data: data,
        token: token,
      });

      if (response.status === 200) {
        Toast.success('获取用户信息成功');
      } else {
        Toast.error(`获取用户信息失败: ${response.status}`);
      }
    } catch (err: unknown) {
      console.error('获取用户信息错误:', err);
      const errorMessage = err instanceof Error ? err.message : '未知错误';
      setError(errorMessage);
      Toast.error('获取用户信息失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ padding: 24 }}>
      <Card>
        <Title heading={3}>API 调试工具</Title>

        <Space style={{ marginTop: 16, marginBottom: 24 }}>
          <Button type="primary" loading={loading} onClick={testLogin}>
            测试登录API (axios)
          </Button>

          <Button loading={loading} onClick={testDirectFetch}>
            测试登录API (fetch)
          </Button>

          <Button loading={loading} onClick={testCurrentUser}>
            测试获取用户信息
          </Button>
        </Space>

        {response && (
          <Card style={{ marginTop: 16, backgroundColor: '#f8f9fa' }}>
            <Title heading={4}>响应数据:</Title>
            <pre
              style={{
                background: '#fff',
                padding: 12,
                borderRadius: 4,
                overflow: 'auto',
                fontSize: 12,
              }}
            >
              {JSON.stringify(response, null, 2)}
            </pre>
          </Card>
        )}

        {error && (
          <Card style={{ marginTop: 16, backgroundColor: '#fff2f0', borderColor: '#ffccc7' }}>
            <Title heading={4} style={{ color: '#ff4d4f' }}>
              错误信息:
            </Title>
            <Text type="danger">{error}</Text>
          </Card>
        )}
      </Card>
    </div>
  );
};

export default ApiTest;
