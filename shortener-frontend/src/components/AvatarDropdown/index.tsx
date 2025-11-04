import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Avatar, Dropdown } from '@douyinfe/semi-ui';
import { Toast } from '@/utils/notification';
import { IconUser, IconSetting, IconExit } from '@douyinfe/semi-icons';

export interface AvatarDropdownProps {
  currentUser?: {
    name?: string;
    avatar?: string;
  };
  onLogout?: () => void;
}

const AvatarDropdown: React.FC<AvatarDropdownProps> = ({
  currentUser,
  onLogout
}) => {
  const navigate = useNavigate();

  const handleLogout = async () => {
    if (onLogout) {
      await onLogout();
    }

    // 跳转到登录页面
    const currentPath = window.location.pathname + window.location.search;
    const loginUrl = `/account/login?redirect=${encodeURIComponent(currentPath)}`;
    navigate(loginUrl);
  };

  const menuItems = [
    {
      node: 'item' as const,
      name: '个人设置',
      icon: <IconSetting />,
      onClick: () => {
        // 这里可以添加个人设置页面的导航
        Toast.info('个人设置功能开发中');
      },
    },
    {
      node: 'divider' as const,
    },
    {
      node: 'item' as const,
      name: '退出登录',
      icon: <IconExit />,
      onClick: handleLogout,
    },
  ];

  return (
    <Dropdown
      trigger="click"
      position="bottomRight"
      menu={menuItems}
    >
      <div style={{ cursor: 'pointer', display: 'flex', alignItems: 'center', gap: 8 }}>
        <Avatar
          size="small"
          color="blue"
          src={currentUser?.avatar}
        >
          {currentUser?.name ? currentUser.name.charAt(0).toUpperCase() : <IconUser />}
        </Avatar>
      </div>
    </Dropdown>
  );
};

export default AvatarDropdown;
