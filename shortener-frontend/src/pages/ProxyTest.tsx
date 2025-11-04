import React, { useState } from 'react';
import { Card, Button, Typography, Space, Divider } from '@douyinfe/semi-ui';
import { Toast } from '@/utils/notification';
import axios, { AxiosError } from 'axios';

const { Title, Text, Paragraph } = Typography;

const ProxyTest: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<Array<{ name: string; data: unknown; timestamp: string }>>(
    [],
  );

  const addResult = (name: string, data: unknown) => {
    setResults((prev) => [...prev, { name, data, timestamp: new Date().toLocaleTimeString() }]);
  };

  const testDirectRequest = async () => {
    setLoading(true);
    try {
      const response = await axios.post('https://dwz.asfd.cn/api/account/login', {
        username: 'test',
        password: 'test',
      });
      addResult('直接请求 (应该失败 - CORS)', response.data);
      Toast.success('直接请求成功');
    } catch (error: unknown) {
      const axiosError = error as any;
      addResult('直接请求 (CORS 错误)', {
        message: axiosError.message,
        status: axiosError.response?.status,
        data: axiosError.response?.data,
      });
      Toast.error('直接请求失败 (预期的 CORS 错误)');
    } finally {
      setLoading(false);
    }
  };

  const testProxyRequest = async () => {
    setLoading(true);
    try {
      const response = await axios.post('/api/account/login', {
        username: 'test',
        password: 'test',
      });
      addResult('代理请求 (应该成功)', response.data);
      Toast.success('代理请求成功');
    } catch (error: unknown) {
      const axiosError = error as any;
      addResult('代理请求 (错误)', {
        message: axiosError.message,
        status: axiosError.response?.status,
        data: axiosError.response?.data,
      });
      Toast.error('代理请求失败');
    } finally {
      setLoading(false);
    }
  };

  const testProxyHealth = async () => {
    setLoading(true);
    try {
      // 测试一个简单的 GET 请求
      const response = await axios.get('/api/users/current');
      addResult('代理健康检查', response.data);
      Toast.success('代理健康检查成功');
    } catch (error: unknown) {
      const axiosError = error as any;
      addResult('代理健康检查 (错误)', {
        message: axiosError.message,
        status: axiosError.response?.status,
        data: axiosError.response?.data,
      });
      Toast.error('代理健康检查失败');
    } finally {
      setLoading(false);
    }
  };

  const testCurrentUserWithToken = async () => {
    setLoading(true);
    try {
      // 先获取 token
      const token = localStorage.getItem('token');
      if (!token) {
        addResult('获取用户信息 (无 Token)', { error: '没有找到 token，请先登录' });
        Toast.error('请先登录');
        return;
      }

      // 使用 token 请求用户信息
      const response = await axios.get('/api/users/current', {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      addResult('获取用户信息 (带 Token)', response.data);
      Toast.success('获取用户信息成功');
    } catch (error: unknown) {
      const axiosError = error as AxiosError;
      addResult('获取用户信息 (错误)', {
        message: axiosError.message,
        status: axiosError.response?.status,
        data: axiosError.response?.data,
      });
      Toast.error('获取用户信息失败');
    } finally {
      setLoading(false);
    }
  };

  const clearResults = () => {
    setResults([]);
  };

  return (
    <div style={{ padding: 24 }}>
      <Card>
        <Title heading={2}>代理测试</Title>
        <Text type="secondary">测试 Vite 代理配置是否正确工作</Text>

        <Divider />

        <Paragraph>
          <Text strong>测试说明：</Text>
          <br />• <Text type="success">代理请求</Text>：通过 Vite 代理发送请求，应该成功
          <br />• <Text type="danger">直接请求</Text>：直接向远程服务器发送请求，会被 CORS 阻止
          <br />• <Text type="tertiary">健康检查</Text>：测试代理服务器连通性
        </Paragraph>

        <Divider />

        <Space wrap>
          <Button type="primary" onClick={testProxyRequest} loading={loading}>
            测试代理请求
          </Button>

          <Button onClick={testProxyHealth} loading={loading}>
            代理健康检查
          </Button>

          <Button onClick={testCurrentUserWithToken} loading={loading}>
            测试获取用户信息
          </Button>

          <Button type="danger" onClick={testDirectRequest} loading={loading}>
            测试直接请求 (会失败)
          </Button>

          <Button type="tertiary" onClick={clearResults}>
            清空结果
          </Button>
        </Space>

        <Divider />

        <div style={{ maxHeight: 400, overflow: 'auto' }}>
          <Title heading={4}>测试结果</Title>
          {results.length === 0 ? (
            <Text type="tertiary">暂无测试结果</Text>
          ) : (
            results.map((result, index) => (
              <Card
                key={index}
                style={{ marginBottom: 8 }}
                title={`${result.name} - ${result.timestamp}`}
              >
                <pre
                  style={{
                    fontSize: 12,
                    background: '#f8f9fa',
                    padding: 8,
                    borderRadius: 4,
                    overflow: 'auto',
                    maxHeight: 200,
                  }}
                >
                  {JSON.stringify(result.data, null, 2)}
                </pre>
              </Card>
            ))
          )}
        </div>
      </Card>
    </div>
  );
};

export default ProxyTest;
