/**
 * @name Semi Design 主题配置
 * @description 配置 Semi Design 的主题变量和样式
 */

export interface SemiThemeConfig {
  palette: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    danger: string;
    info: string;
  };
  typography: {
    fontFamily: string;
    fontSize: {
      small: string;
      regular: string;
      large: string;
      extraLarge: string;
    };
  };
  spacing: {
    base: number;
    small: number;
    medium: number;
    large: number;
    extraLarge: number;
  };
  borderRadius: {
    small: string;
    medium: string;
    large: string;
    extraLarge: string;
  };
  shadow: {
    small: string;
    medium: string;
    large: string;
    extraLarge: string;
  };
}

// Semi Design 主题令牌配置
export const semiDesignTokens = {
  // 颜色配置 - 保持与 Ant Design 相似的主色调
  '--semi-color-primary': '#1890ff',
  '--semi-color-primary-hover': '#40a9ff',
  '--semi-color-primary-active': '#096dd9',
  '--semi-color-secondary': '#722ed1',
  '--semi-color-success': '#52c41a',
  '--semi-color-warning': '#faad14',
  '--semi-color-danger': '#f5222d',
  '--semi-color-info': '#1890ff',

  // 字体配置
  '--semi-font-size-small': '12px',
  '--semi-font-size-regular': '14px',
  '--semi-font-size-large': '16px',
  '--semi-font-size-extra-large': '20px',

  // 间距配置
  '--semi-spacing-base': '8px',
  '--semi-spacing-tight': '4px',
  '--semi-spacing-loose': '12px',
  '--semi-spacing-extra-tight': '2px',
  '--semi-spacing-extra-loose': '16px',

  // 圆角配置
  '--semi-border-radius-small': '2px',
  '--semi-border-radius-medium': '4px',
  '--semi-border-radius-large': '6px',
  '--semi-border-radius-extra-large': '8px',

  // 阴影配置
  '--semi-shadow-small': '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  '--semi-shadow-medium': '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
  '--semi-shadow-large': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
  '--semi-shadow-extra-large': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',

  // 布局配置
  '--semi-layout-header-height': '64px',
  '--semi-layout-sidebar-width': '256px',
  '--semi-layout-sidebar-collapsed-width': '64px',

  // 响应式断点
  '--semi-breakpoint-xs': '480px',
  '--semi-breakpoint-sm': '576px',
  '--semi-breakpoint-md': '768px',
  '--semi-breakpoint-lg': '992px',
  '--semi-breakpoint-xl': '1200px',
  '--semi-breakpoint-xxl': '1600px',
};

export const semiTheme: SemiThemeConfig = {
  // 调色板配置 - 保持与 Ant Design 相似的主色调
  palette: {
    primary: '#1890ff',
    secondary: '#722ed1',
    success: '#52c41a',
    warning: '#faad14',
    danger: '#f5222d',
    info: '#1890ff',
  },

  // 字体配置
  typography: {
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"',
    fontSize: {
      small: '12px',
      regular: '14px',
      large: '16px',
      extraLarge: '20px',
    },
  },

  // 间距配置
  spacing: {
    base: 8,
    small: 4,
    medium: 12,
    large: 16,
    extraLarge: 24,
  },

  // 圆角配置
  borderRadius: {
    small: '2px',
    medium: '4px',
    large: '6px',
    extraLarge: '8px',
  },

  // 阴影配置
  shadow: {
    small: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    medium: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
    large: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
    extraLarge: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
  },
};

/**
 * 应用主题令牌到 DOM
 */
export const applyThemeTokens = () => {
  const root = document.documentElement;
  Object.entries(semiDesignTokens).forEach(([key, value]) => {
    root.style.setProperty(key, value);
  });
};

export default semiTheme;
