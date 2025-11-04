import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Empty } from '@douyinfe/semi-ui';

const NotFoundPage: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="error-page">
      <Empty
        image={<div style={{ fontSize: 64 }}>404</div>}
        title="页面不存在"
        description="抱歉，您访问的页面不存在"
      />
      <div style={{ marginTop: 24 }}>
        <Button
          type="primary"
          onClick={() => navigate('/')}
        >
          返回首页
        </Button>
      </div>
    </div>
  );
};

export default NotFoundPage;
