import React, { Suspense } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { Spin } from '@douyinfe/semi-ui-19';
import MainLayout from './layouts/MainLayout';

// 懒加载页面组件以实现代码分割
const Login = React.lazy(() => import('./pages/Account/Login'));
const Dashboard = React.lazy(() => import('./pages/Dashboard'));
const Shortener = React.lazy(() => import('./pages/Shortener'));
const History = React.lazy(() => import('./pages/History'));
const ApiTest = React.lazy(() => import('./pages/ApiTest'));
const ProxyTest = React.lazy(() => import('./pages/ProxyTest'));
const NotFound = React.lazy(() => import('./pages/404'));

// 加载中组件
const PageLoading: React.FC = () => (
  <div
    style={{
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      height: '200px',
    }}
  >
    <Spin size="large" />
  </div>
);

const App: React.FC = () => {
  return (
    <Suspense fallback={<PageLoading />}>
      <Routes>
        {/* 登录页面，不使用主布局 */}
        <Route path="/account/login" element={<Login />} />

        {/* 主应用路由，使用主布局 */}
        <Route path="/" element={<MainLayout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route
            path="dashboard"
            element={
              <Suspense fallback={<PageLoading />}>
                <Dashboard />
              </Suspense>
            }
          />
          <Route
            path="shortens"
            element={
              <Suspense fallback={<PageLoading />}>
                <Shortener />
              </Suspense>
            }
          />
          <Route
            path="histories"
            element={
              <Suspense fallback={<PageLoading />}>
                <History />
              </Suspense>
            }
          />
          <Route
            path="api-test"
            element={
              <Suspense fallback={<PageLoading />}>
                <ApiTest />
              </Suspense>
            }
          />
          <Route
            path="proxy-test"
            element={
              <Suspense fallback={<PageLoading />}>
                <ProxyTest />
              </Suspense>
            }
          />
        </Route>

        {/* 404 页面 */}
        <Route path="*" element={<NotFound />} />
      </Routes>
    </Suspense>
  );
};

export default App;
