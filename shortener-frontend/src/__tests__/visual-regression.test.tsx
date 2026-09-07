import React from 'react';
import { render } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BrowserRouter } from 'react-router-dom';

// Import components to test
import Dashboard from '../pages/Dashboard';
import Shortener from '../pages/Shortener';
import History from '../pages/History';
import MainLayout from '../layouts/MainLayout';

// Mock Semi UI components for consistent rendering
vi.mock('@douyinfe/semi-ui-19', () => ({
  Card: ({ children, title, extra }: any) => (
    <div className="semi-card" data-testid="card">
      {title && <div className="semi-card-title">{title}</div>}
      {extra && <div className="semi-card-extra">{extra}</div>}
      <div className="semi-card-content">{children}</div>
    </div>
  ),
  Button: ({ children, type, theme, size, icon, loading: _loading, ...props }: any) => (
    <button
      className={`semi-button semi-button-${type || 'default'} semi-button-${theme || 'solid'} semi-button-${size || 'default'}`}
      {...props}
    >
      {icon}
      {children}
    </button>
  ),
  Modal: ({ visible, title, children }: any) =>
    visible ? (
      <div className="semi-modal">
        <div className="semi-modal-title">{title}</div>
        <div className="semi-modal-content">{children}</div>
      </div>
    ) : null,
  Form: Object.assign(({ children }: any) => <form className="semi-form">{children}</form>, {
    Input: ({ label }: any) => <div className="semi-form-field">{label}</div>,
    TextArea: ({ label }: any) => <div className="semi-form-field">{label}</div>,
  }),
  Layout: {
    Header: ({ children, style }: any) => (
      <header className="semi-layout-header" style={style}>
        {children}
      </header>
    ),
    Sider: ({ children, style }: any) => (
      <aside className="semi-layout-sider" style={style}>
        {children}
      </aside>
    ),
    Content: ({ children, style }: any) => (
      <main className="semi-layout-content" style={style}>
        {children}
      </main>
    ),
    Footer: ({ children, style }: any) => (
      <footer className="semi-layout-footer" style={style}>
        {children}
      </footer>
    ),
  },
  Nav: ({ items, selectedKeys, mode, isCollapsed }: any) => (
    <nav className={`semi-nav semi-nav-${mode} ${isCollapsed ? 'semi-nav-collapsed' : ''}`}>
      {items?.map((item: any) => (
        <div
          key={item.itemKey}
          className={`semi-nav-item ${selectedKeys?.includes(item.itemKey) ? 'semi-nav-item-selected' : ''}`}
        >
          {item.icon}
          {!isCollapsed && <span>{item.text}</span>}
        </div>
      ))}
    </nav>
  ),
  Typography: Object.assign(
    ({ children, heading, style }: any) => (
      <h1 className={`semi-typography-title semi-typography-h${heading || 1}`} style={style}>
        {children}
      </h1>
    ),
    {
      Title: ({ children, heading, style }: any) => (
        <h1 className={`semi-typography-title semi-typography-h${heading || 1}`} style={style}>
          {children}
        </h1>
      ),
      Text: ({ children, type }: any) => (
        <span className={`semi-typography-text ${type ? `semi-typography-${type}` : ''}`}>
          {children}
        </span>
      ),
      Paragraph: ({ children }: any) => <p className="semi-typography-paragraph">{children}</p>,
    },
  ),
  Space: ({ children, direction = 'horizontal', size = 'small' }: any) => (
    <div className={`semi-space semi-space-${direction} semi-space-${size}`}>{children}</div>
  ),
  Spin: ({ children, spinning = false, size = 'default' }: any) => (
    <div className={`semi-spin semi-spin-${size} ${spinning ? 'semi-spin-spinning' : ''}`}>
      {children}
    </div>
  ),
  SideSheet: ({ visible, children, placement = 'right' }: any) =>
    visible ? <div className={`semi-sidesheet semi-sidesheet-${placement}`}>{children}</div> : null,
  Toast: {
    info: vi.fn(),
    update: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock Semi Icons
vi.mock('@douyinfe/semi-icons', () => ({
  IconHome: () => <span className="semi-icon semi-icon-home">🏠</span>,
  IconLink: () => <span className="semi-icon semi-icon-link">🔗</span>,
  IconHistogram: () => <span className="semi-icon semi-icon-histogram">📊</span>,
  IconMenu: () => <span className="semi-icon semi-icon-menu">☰</span>,
  IconPlus: () => <span className="semi-icon semi-icon-plus">➕</span>,
  IconCopy: () => <span className="semi-icon semi-icon-copy">⧉</span>,
  IconGithubLogo: () => <span className="semi-icon semi-icon-github">📱</span>,
  IconGitlabLogo: () => <span className="semi-icon semi-icon-gitlab">🦊</span>,
  IconSearch: () => <span className="semi-icon semi-icon-search">🔍</span>,
  IconRefresh: () => <span className="semi-icon semi-icon-refresh">🔄</span>,
}));

// Mock components
vi.mock('../components', () => ({
  AvatarDropdown: ({ currentUser }: any) => (
    <div className="avatar-dropdown">
      <span className="avatar-text">{currentUser?.name || 'Guest'}</span>
    </div>
  ),
  Footer: () => (
    <div className="footer-component">
      <span>© 2024 Shortener. All rights reserved.</span>
    </div>
  ),
  SemiTable: ({ headerTitle, columns }: any) => (
    <div className="semi-table-wrapper">
      <div className="semi-table-header">{headerTitle}</div>
      <table className="semi-table">
        <thead>
          <tr>
            {columns?.map((col: any) => (
              <th key={col.key || col.dataIndex} className="semi-table-th">
                {col.title}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          <tr>
            <td className="semi-table-td" colSpan={columns?.length || 1}>
              Sample Data
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  ),
  SemiModalForm: ({ visible, title, children }: any) =>
    visible ? (
      <div className="semi-modal-form">
        <div className="semi-modal-title">{title}</div>
        <div className="semi-modal-content">{children}</div>
      </div>
    ) : null,
}));

// Mock notification
vi.mock('@/utils/notification', () => ({
  Toast: {
    info: vi.fn(),
    update: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock services
vi.mock('@/services/shortener/shorten', () => ({
  getShortens: vi.fn().mockResolvedValue({ data: [], success: true, meta: { total: 0 } }),
  addShorten: vi.fn().mockResolvedValue({ success: true }),
  updateShorten: vi.fn().mockResolvedValue({ success: true }),
  deleteShorten: vi.fn().mockResolvedValue({ success: true }),
}));

vi.mock('@/services/shortener/history', () => ({
  getHistories: vi.fn().mockResolvedValue({ data: [], success: true, meta: { total: 0 } }),
  deleteHistories: vi.fn().mockResolvedValue({ success: true }),
}));

vi.mock('@/services/shortener/cache', () => ({
  refreshCache: vi.fn().mockResolvedValue({ cleared_keys: 0, warmed_urls: 0 }),
}));

// Mock hooks
vi.mock('../hooks/useAuth', () => ({
  useAuth: () => ({
    currentUser: { id: 1, name: 'Test User' },
    loading: false,
    isAuth: true,
    logout: vi.fn(),
  }),
}));

// Mock react-router-dom
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: '/dashboard' }),
    Outlet: () => <div className="router-outlet">Page Content</div>,
  };
});

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

// Helper function to get component structure
const getComponentStructure = (container: HTMLElement) => {
  const getElementInfo = (element: Element): any => {
    const info: any = {
      tagName: element.tagName.toLowerCase(),
      className: element.className || undefined,
      textContent: element.textContent?.trim() || undefined,
    };

    // Only include non-empty properties
    Object.keys(info).forEach((key) => {
      if (!info[key]) delete info[key];
    });

    const children = Array.from(element.children).map(getElementInfo);
    if (children.length > 0) {
      info.children = children;
    }

    return info;
  };

  return getElementInfo(container.firstElementChild || container);
};

describe('Visual Regression Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Dashboard Component', () => {
    it('renders with consistent structure and styling', () => {
      const { container } = renderWithRouter(<Dashboard />);
      const structure = getComponentStructure(container);

      expect(structure).toMatchSnapshot('dashboard-structure');

      // Verify key visual elements
      expect(container.querySelector('.semi-card')).toBeInTheDocument();
      expect(container.querySelector('.semi-typography-title')).toBeInTheDocument();
      expect(container.querySelector('.semi-typography-text')).toBeInTheDocument();
    });

    it('maintains consistent layout structure', () => {
      const { container } = renderWithRouter(<Dashboard />);

      // Check for proper card structure
      const card = container.querySelector('.semi-card');
      expect(card).toBeInTheDocument();

      const cardContent = card?.querySelector('.semi-card-content');
      expect(cardContent).toBeInTheDocument();

      // Verify title hierarchy
      const title = container.querySelector('.semi-typography-title');
      expect(title).toHaveClass('semi-typography-h3');
    });
  });

  describe('Shortener Component', () => {
    it('renders table with consistent styling', () => {
      const { container } = renderWithRouter(<Shortener />);
      const structure = getComponentStructure(container);

      expect(structure).toMatchSnapshot('shortener-structure');

      // Verify table elements
      expect(container.querySelector('.semi-table-wrapper')).toBeInTheDocument();
      expect(container.querySelector('.semi-table')).toBeInTheDocument();
      expect(container.querySelector('.semi-table-header')).toBeInTheDocument();
    });

    it('maintains consistent button styling', () => {
      const { container } = renderWithRouter(<Shortener />);

      const buttons = container.querySelectorAll('.semi-button');
      buttons.forEach((button) => {
        expect(button).toHaveClass('semi-button');
        // Should have type and theme classes
        expect(button.className).toMatch(/semi-button-(default|primary|secondary)/);
        expect(button.className).toMatch(/semi-button-(solid|borderless|light)/);
      });
    });
  });

  describe('History Component', () => {
    it('renders with consistent table layout', () => {
      const { container } = renderWithRouter(<History />);
      const structure = getComponentStructure(container);

      expect(structure).toMatchSnapshot('history-structure');

      // Verify table structure
      expect(container.querySelector('.semi-table-wrapper')).toBeInTheDocument();
      expect(container.querySelector('.semi-table-header')).toBeInTheDocument();
    });
  });

  describe('MainLayout Component', () => {
    it('renders layout with consistent structure', () => {
      const { container } = renderWithRouter(<MainLayout />);
      const structure = getComponentStructure(container);

      expect(structure).toMatchSnapshot('main-layout-structure');

      // Verify layout components - MainLayout uses custom divs, not Semi Layout
      expect(container.querySelector('.semi-nav')).toBeInTheDocument();
      expect(container.querySelector('.footer-component')).toBeInTheDocument();
      expect(container.querySelector('.avatar-dropdown')).toBeInTheDocument();
    });

    it('maintains consistent navigation styling', () => {
      const { container } = renderWithRouter(<MainLayout />);

      const nav = container.querySelector('.semi-nav');
      expect(nav).toBeInTheDocument();
      expect(nav).toHaveClass('semi-nav-vertical');

      const navItems = container.querySelectorAll('.semi-nav-item');
      expect(navItems.length).toBeGreaterThan(0);

      navItems.forEach((item) => {
        expect(item).toHaveClass('semi-nav-item');
      });
    });

    it('handles collapsed state correctly', () => {
      // Mock window size for mobile
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: 600,
      });

      const { container } = renderWithRouter(<MainLayout />);

      // 移动端：侧边栏收进抽屉，抽屉关闭时不渲染 nav
      expect(container.querySelector('.semi-nav')).not.toBeInTheDocument();
    });
  });

  describe('Icon Consistency', () => {
    it('renders icons with consistent styling', () => {
      const { container } = renderWithRouter(<MainLayout />);

      const icons = container.querySelectorAll('.semi-icon');
      icons.forEach((icon) => {
        expect(icon).toHaveClass('semi-icon');
        // Should have specific icon class
        expect(icon.className).toMatch(/semi-icon-(home|link|histogram|menu)/);
      });
    });
  });

  describe('Typography Consistency', () => {
    it('maintains consistent text styling across components', () => {
      const { container } = renderWithRouter(<Dashboard />);

      const title = container.querySelector('.semi-typography-title');
      expect(title).toHaveClass('semi-typography-title');

      const text = container.querySelector('.semi-typography-text');
      expect(text).toHaveClass('semi-typography-text');
    });
  });

  describe('Color and Theme Consistency', () => {
    it('applies consistent Semi Design theme classes', () => {
      const { container } = renderWithRouter(<Shortener />);

      // Check for Semi Design class patterns
      const semiElements = container.querySelectorAll('[class*="semi-"]');
      expect(semiElements.length).toBeGreaterThan(0);

      semiElements.forEach((element) => {
        // Should follow Semi Design naming convention
        expect(element.className).toMatch(/semi-\w+/);
      });
    });
  });

  describe('Responsive Layout', () => {
    it('maintains layout integrity at different screen sizes', () => {
      // Test desktop layout
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: 1200,
      });

      const { container: desktopContainer } = renderWithRouter(<MainLayout />);
      const desktopStructure = getComponentStructure(desktopContainer);

      // Test mobile layout
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: 600,
      });

      const { container: mobileContainer } = renderWithRouter(<MainLayout />);
      const mobileStructure = getComponentStructure(mobileContainer);

      // 桌面端渲染 nav；移动端 nav 收进关闭的抽屉不渲染
      expect(desktopContainer.querySelector('.semi-nav')).toBeInTheDocument();
      expect(mobileContainer.querySelector('.semi-nav')).not.toBeInTheDocument();

      // Structures should be consistent
      expect(desktopStructure.tagName).toBe(mobileStructure.tagName);
    });
  });

  describe('Component State Consistency', () => {
    it('maintains visual consistency across different states', () => {
      // Test loading state
      const { container: loadingContainer } = renderWithRouter(<Shortener />);

      // Test loaded state
      const { container: loadedContainer } = renderWithRouter(<Shortener />);

      // Both should have consistent base structure
      expect(loadingContainer.querySelector('.semi-table-wrapper')).toBeInTheDocument();
      expect(loadedContainer.querySelector('.semi-table-wrapper')).toBeInTheDocument();
    });
  });
});
