import React from 'react';
import { Card, Typography } from '@douyinfe/semi-ui-19';

const { Title, Text } = Typography;

const Dashboard: React.FC = () => {
  return (
    <Card
      data-testid="card"
      style={{
        borderRadius: 8,
        background:
          'linear-gradient(75deg, var(--semi-color-bg-0) 0%, var(--semi-color-fill-0) 100%)',
        border: '1px solid var(--semi-color-border)',
      }}
      bodyStyle={{
        padding: 32,
      }}
    >
      <div
        data-testid="card-content"
        style={{
          backgroundPosition: '100% -30%',
          backgroundRepeat: 'no-repeat',
          backgroundSize: '274px auto',
        }}
      >
        <Title
          data-testid="title"
          heading={3}
          style={{
            fontSize: '20px',
            color: 'var(--semi-color-text-0)',
            marginBottom: 0,
          }}
        >
          欢迎使用 Shortener 短网址生成器
        </Title>
        <Text
          data-testid="text"
          style={{
            fontSize: '14px',
            color: 'var(--semi-color-text-1)',
            lineHeight: '22px',
            marginTop: 16,
            marginBottom: 32,
            width: '65%',
            display: 'block',
          }}
        >
          Shortener 是一个使用 Rust 语言开发的短网址生成器，UI 框架使用 Semi Design。
        </Text>
      </div>
    </Card>
  );
};

export default Dashboard;
