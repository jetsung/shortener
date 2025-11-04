import { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { currentUser as getCurrentUser, logout as logoutApi } from '../services/shortener/account';
import { Toast } from '@/utils/notification';
import { isAuthenticated, clearAuth } from '@/utils/api';

interface CurrentUser {
  name?: string;
  avatar?: string;
}

export const useAuth = () => {
  const [currentUser, setCurrentUser] = useState<CurrentUser | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();
  const location = useLocation();

  const checkAuth = async () => {
    try {
      if (!isAuthenticated()) {
        // 如果没有token，跳转到登录页
        const currentPath = location.pathname + location.search;
        navigate(`/account/login?redirect=${encodeURIComponent(currentPath)}`);
        return;
      }

      // 如果已经有用户信息，就不需要重新获取
      if (currentUser) {
        setLoading(false);
        return;
      }

      // 调用API获取用户信息
      const userInfo = await getCurrentUser() as { name?: string };
      if (userInfo && userInfo.name) {
        setCurrentUser({
          name: userInfo.name,
          avatar: undefined, // API暂时不返回头像
        });
      } else {
        // 如果获取用户信息失败，但有 token，可能是 API 问题，先设置一个默认用户
        console.warn('Failed to get user info, but token exists. Using default user.');
        setCurrentUser({
          name: 'User', // 默认用户名
          avatar: undefined,
        });
      }
    } catch (error) {
      console.error('Auth check failed:', error);
      // 如果有 token 但 API 调用失败，可能是网络问题，不要立即清除认证
      if (isAuthenticated()) {
        console.warn('API call failed but token exists. Using default user.');
        setCurrentUser({
          name: 'User', // 默认用户名
          avatar: undefined,
        });
      } else {
        // 只有在没有 token 时才跳转到登录页
        const currentPath = location.pathname + location.search;
        navigate(`/account/login?redirect=${encodeURIComponent(currentPath)}`);
      }
    } finally {
      setLoading(false);
    }
  };

  const logout = async () => {
    try {
      // 调用API进行登出
      await logoutApi();
      // 清除本地状态和存储
      setCurrentUser(null);
      clearAuth();
      Toast.success('已成功退出登录');
    } catch (error) {
      console.error('Logout failed:', error);
      // 即使API调用失败，也要清除本地状态
      setCurrentUser(null);
      clearAuth();
      Toast.warning('退出登录');
    }
  };

  useEffect(() => {
    // 只在组件首次挂载时检查认证
    checkAuth();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // 移除依赖，避免路径变化时重复检查

  return {
    currentUser,
    loading,
    logout,
    checkAuth,
  };
};
