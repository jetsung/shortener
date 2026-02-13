import React, { useState, useEffect, memo } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { Nav, Button, Typography, Spin, SideSheet } from '@douyinfe/semi-ui-19';
import { IconHome, IconLink, IconHistogram, IconMenu } from '@douyinfe/semi-icons';
import { AvatarDropdown, Footer } from '../components';
import { useAuth } from '../hooks/useAuth';
import { usePerformance } from '../hooks/usePerformance';

const { Title } = Typography;

const MainLayout: React.FC = memo(() => {
  const [collapsed, setCollapsed] = useState(false);
  const [isMobile, setIsMobile] = useState(false);
  const [mobileDrawerVisible, setMobileDrawerVisible] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const { currentUser, loading, isAuth, logout: handleLogout } = useAuth();
  usePerformance('MainLayout');

  // 检测屏幕尺寸的 useEffect（必须在条件渲染之前）
  useEffect(() => {
    const checkScreenSize = () => {
      const mobile = window.innerWidth < 768;
      setIsMobile(mobile);
      if (mobile) {
        setCollapsed(true);
      }
    };

    checkScreenSize();
    window.addEventListener('resize', checkScreenSize);
    return () => window.removeEventListener('resize', checkScreenSize);
  }, []);

  // 未登录或加载中，显示加载状态
  if (loading || !isAuth) {
    return (
      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          height: '100vh',
        }}
      >
        <Spin size="large" />
      </div>
    );
  }

  // 导航菜单项
  const navItems = [
    {
      itemKey: '/dashboard',
      text: '仪表盘',
      icon: <IconHome />,
    },
    {
      itemKey: '/shortens',
      text: '短址管理',
      icon: <IconLink />,
    },
    {
      itemKey: '/histories',
      text: '日志管理',
      icon: <IconHistogram />,
    },
  ];

  const handleNavSelect = (data: { itemKey: string | number }) => {
    navigate(String(data.itemKey));
    // 在移动端选择菜单后关闭抽屉
    if (isMobile) {
      setMobileDrawerVisible(false);
    }
  };

  const handleMenuToggle = () => {
    if (isMobile) {
      setMobileDrawerVisible(!mobileDrawerVisible);
    } else {
      setCollapsed(!collapsed);
    }
  };

  // 渲染侧边栏内容
  const renderSidebarContent = (isInDrawer = false) => (
    <div
      style={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Logo 区域 */}
      <div
        style={{
          height: 64,
          display: 'flex',
          alignItems: 'center',
          justifyContent: collapsed && !isInDrawer ? 'center' : 'flex-start',
          padding: collapsed && !isInDrawer ? 0 : '0 24px',
          borderBottom: '1px solid var(--semi-color-border)',
          flexShrink: 0,
        }}
      >
        {(!collapsed || isInDrawer) && (
          <Title heading={4} style={{ margin: 0, color: 'var(--semi-color-primary)' }}>
            Shortener
          </Title>
        )}
        {collapsed && !isInDrawer && (
          <div
            style={{
              width: 32,
              height: 32,
              backgroundColor: 'var(--semi-color-primary)',
              borderRadius: 6,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'white',
              fontWeight: 'bold',
            }}
          >
            S
          </div>
        )}
      </div>

      {/* 导航菜单 */}
      <div
        style={{
          flex: 1,
          overflow: 'hidden',
          paddingTop: 16,
        }}
      >
        <Nav
          items={navItems}
          selectedKeys={[location.pathname]}
          onSelect={handleNavSelect}
          mode="vertical"
          isCollapsed={collapsed && !isInDrawer}
        />
      </div>
    </div>
  );

  return (
    <div
      style={{
        height: '100vh',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'row',
      }}
    >
      {/* 桌面端固定侧边栏 */}
      {!isMobile && (
        <div
          style={{
            width: collapsed ? 64 : 240,
            height: '100vh',
            backgroundColor: 'var(--semi-color-bg-1)',
            borderRight: '1px solid var(--semi-color-border)',
            position: 'fixed',
            left: 0,
            top: 0,
            zIndex: 100,
            transition: 'width 0.2s ease',
            overflow: 'hidden',
          }}
        >
          {renderSidebarContent()}
        </div>
      )}

      {/* 移动端抽屉菜单 */}
      {isMobile && (
        <SideSheet
          title={null}
          visible={mobileDrawerVisible}
          onCancel={() => setMobileDrawerVisible(false)}
          placement="left"
          width={280}
          bodyStyle={{ padding: 0 }}
          headerStyle={{ display: 'none' }}
        >
          <div style={{ backgroundColor: 'var(--semi-color-bg-1)', height: '100%' }}>
            {renderSidebarContent(true)}
          </div>
        </SideSheet>
      )}

      {/* 主内容区域 */}
      <div
        style={{
          flex: 1,
          height: '100vh',
          marginLeft: isMobile ? 0 : collapsed ? 64 : 240,
          display: 'flex',
          flexDirection: 'column',
          transition: 'margin-left 0.2s ease',
        }}
      >
        {/* 固定顶部导航栏 */}
        <div
          style={{
            height: 64,
            backgroundColor: 'var(--semi-color-bg-1)',
            borderBottom: '1px solid var(--semi-color-border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: isMobile ? '0 16px' : '0 24px',
            position: 'fixed',
            top: 0,
            right: 0,
            left: isMobile ? 0 : collapsed ? 64 : 240,
            zIndex: 99,
            transition: 'left 0.2s ease',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <Button
              theme="borderless"
              icon={<IconMenu />}
              onClick={handleMenuToggle}
              style={{
                color: 'var(--semi-color-text-0)',
              }}
            />

            {/* 移动端显示标题 */}
            {isMobile && (
              <Title heading={5} style={{ margin: 0, color: 'var(--semi-color-primary)' }}>
                Shortener
              </Title>
            )}
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <AvatarDropdown currentUser={currentUser || undefined} onLogout={handleLogout} />
          </div>
        </div>

        {/* 主内容区域 - 铺满整个区域 */}
        <div
          style={{
            flex: 1,
            marginTop: 64, // 为固定的顶部导航留出空间
            backgroundColor: 'var(--semi-color-bg-0)',
            overflow: 'auto',
            width: '100%',
          }}
        >
          <div
            style={{
              width: '100%',
              minHeight: '100%',
              padding: isMobile ? 16 : 24,
            }}
          >
            <Outlet />
          </div>
        </div>

        {/* 页脚 */}
        <div
          style={{
            backgroundColor: 'var(--semi-color-bg-1)',
            borderTop: '1px solid var(--semi-color-border)',
            flexShrink: 0,
          }}
        >
          <Footer />
        </div>
      </div>
    </div>
  );
});

MainLayout.displayName = 'MainLayout';

export default MainLayout;
