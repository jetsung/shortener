import { useState, useEffect, useRef } from 'react';
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
  const [isAuth, setIsAuth] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const hasCheckedAuth = useRef(false);

  // 检查是否在登录页
  const isLoginPage = () => {
    const path = location.hash ? location.hash.substring(1) : location.pathname;
    return path.includes('/account/login');
  };

  const checkAuth = async () => {
    // 如果在登录页，不检查
    if (isLoginPage()) {
      setLoading(false);
      return;
    }

    // 防止重复检查
    if (hasCheckedAuth.current) {
      setLoading(false);
      return;
    }

    if (!isAuthenticated()) {
      setIsAuth(false);
      const redirect = location.hash
        ? location.hash.substring(1) + location.search
        : location.pathname + location.search;
      navigate(`/account/login?redirect=${encodeURIComponent(redirect)}`, { replace: true });
      setLoading(false);
      return;
    }

    try {
      const userInfo = (await getCurrentUser()) as { name?: string };
      if (userInfo && userInfo.name) {
        setCurrentUser({
          name: userInfo.name,
          avatar: undefined,
        });
        setIsAuth(true);
        hasCheckedAuth.current = true;
      } else {
        setIsAuth(false);
        clearAuth();
        hasCheckedAuth.current = false;
        const redirect = location.hash
          ? location.hash.substring(1) + location.search
          : location.pathname + location.search;
        navigate(`/account/login?redirect=${encodeURIComponent(redirect)}`, { replace: true });
      }
    } catch (error: any) {
      const is401 = error.response?.status === 401 || error.response?.status === 403;
      if (is401) {
        setIsAuth(false);
        clearAuth();
        hasCheckedAuth.current = false;
        const redirect = location.hash
          ? location.hash.substring(1) + location.search
          : location.pathname + location.search;
        navigate(`/account/login?redirect=${encodeURIComponent(redirect)}`, { replace: true });
      } else {
        setIsAuth(true);
        hasCheckedAuth.current = true;
      }
    } finally {
      setLoading(false);
    }
  };

  const logout = async () => {
    try {
      await logoutApi();
    } catch (error) {
      console.error('Logout API failed:', error);
    }
    setCurrentUser(null);
    clearAuth();
    hasCheckedAuth.current = false;
    Toast.success('已成功退出登录');
    navigate(`/account/login`, { replace: true });
  };

  useEffect(() => {
    checkAuth();
  }, []);

  return {
    currentUser,
    loading,
    isAuth,
    logout,
    checkAuth,
  };
};
