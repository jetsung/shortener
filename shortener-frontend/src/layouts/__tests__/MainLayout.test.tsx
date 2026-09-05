import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import MainLayout from '../MainLayout';

// Mock react-router-dom
const mockNavigate = vi.fn();
const mockLocation = { pathname: '/dashboard' };

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useLocation: () => mockLocation,
    Outlet: () => <div data-testid="outlet">Page Content</div>,
  };
});

// Mock Semi UI components
vi.mock('@douyinfe/semi-ui-19', () => ({
  Layout: {
    Header: ({ children, style }: any) => (
      <header style={style} data-testid="header">
        {children}
      </header>
    ),
    Sider: ({ children, style }: any) => (
      <aside style={style} data-testid="sider">
        {children}
      </aside>
    ),
    Content: ({ children, style }: any) => (
      <main style={style} data-testid="content">
        {children}
      </main>
    ),
    Footer: ({ children, style }: any) => (
      <footer style={style} data-testid="footer">
        {children}
      </footer>
    ),
  },
  Nav: ({ items, selectedKeys, onSelect, mode, isCollapsed }: any) => (
    <nav data-testid="nav" data-mode={mode} data-collapsed={isCollapsed}>
      {items?.map((item: any) => (
        <button
          key={item.itemKey}
          onClick={() => onSelect({ itemKey: item.itemKey })}
          data-selected={selectedKeys?.includes(item.itemKey)}
          data-testid={`nav-item-${item.itemKey}`}
        >
          {item.text}
        </button>
      ))}
    </nav>
  ),
  Button: ({ children, onClick, icon, theme, ...props }: any) => (
    <button onClick={onClick} data-theme={theme} {...props}>
      {icon}
      {children}
    </button>
  ),
  Typography: {
    Title: ({ children, heading, style }: any) => (
      <h1 style={style} data-heading={heading} data-testid="title">
        {children}
      </h1>
    ),
  },
  Spin: ({ size }: any) => (
    <div data-testid="spin" data-size={size}>
      Loading...
    </div>
  ),
  SideSheet: ({ visible, children, onCancel, placement, title }: any) =>
    visible ? (
      <div data-testid="side-sheet" data-placement={placement}>
        {title && <div data-testid="side-sheet-title">{title}</div>}
        <button onClick={onCancel} data-testid="side-sheet-close">
          Close
        </button>
        {children}
      </div>
    ) : null,
}));

// Mock Semi Icons
vi.mock('@douyinfe/semi-icons', () => ({
  IconHome: () => <span data-testid="icon-home">🏠</span>,
  IconLink: () => <span data-testid="icon-link">🔗</span>,
  IconHistogram: () => <span data-testid="icon-histogram">📊</span>,
  IconMenu: () => <span data-testid="icon-menu">☰</span>,
}));

// Mock components
vi.mock('../components', () => ({
  AvatarDropdown: ({ currentUser, onLogout }: any) => (
    <div data-testid="avatar-dropdown">
      <span data-testid="current-user">{currentUser?.name || 'Guest'}</span>
      <button onClick={onLogout} data-testid="logout-btn">
        Logout
      </button>
    </div>
  ),
  Footer: () => <div data-testid="footer-component">Footer Content</div>,
}));

vi.mock('../../components', () => ({
  AvatarDropdown: ({
    currentUser,
    onLogout,
  }: {
    currentUser?: { name?: string };
    onLogout?: () => void;
  }) => (
    <div data-testid="avatar-dropdown">
      <span data-testid="current-user">{currentUser?.name || 'Guest'}</span>
      <button onClick={onLogout} data-testid="logout-btn">
        Logout
      </button>
    </div>
  ),
  Footer: () => <div data-testid="footer-component">Footer Content</div>,
}));

// Mock useAuth hook
const mockUseAuth = {
  currentUser: { id: 1, name: 'Test User' },
  loading: false,
  isAuth: true,
  logout: vi.fn(),
};

vi.mock('../../hooks/useAuth', () => ({
  useAuth: () => mockUseAuth,
}));

// Mock window.innerWidth for responsive tests
Object.defineProperty(window, 'innerWidth', {
  writable: true,
  configurable: true,
  value: 1024,
});

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('MainLayout', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.innerWidth = 1024;
    mockUseAuth.loading = false;
    mockUseAuth.isAuth = true;
    mockUseAuth.currentUser = { id: 1, name: 'Test User' };
  });

  it('renders main layout structure', () => {
    renderWithRouter(<MainLayout />);

    expect(screen.getByTestId('nav')).toBeInTheDocument();
    expect(screen.getByTestId('title')).toBeInTheDocument();
    expect(screen.getByTestId('footer-component')).toBeInTheDocument();
    expect(screen.getByTestId('outlet')).toBeInTheDocument();
  });

  it('displays navigation items correctly', () => {
    renderWithRouter(<MainLayout />);

    expect(screen.getByTestId('nav-item-/dashboard')).toBeInTheDocument();
    expect(screen.getByTestId('nav-item-/shortens')).toBeInTheDocument();
    expect(screen.getByTestId('nav-item-/histories')).toBeInTheDocument();

    expect(screen.getByText('仪表盘')).toBeInTheDocument();
    expect(screen.getByText('短址管理')).toBeInTheDocument();
    expect(screen.getByText('日志管理')).toBeInTheDocument();
  });

  it('shows loading state when auth is loading', () => {
    mockUseAuth.loading = true;

    renderWithRouter(<MainLayout />);

    expect(screen.getByTestId('spin')).toBeInTheDocument();
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('handles navigation selection', () => {
    renderWithRouter(<MainLayout />);

    fireEvent.click(screen.getByTestId('nav-item-/shortens'));
    expect(mockNavigate).toHaveBeenCalledWith('/shortens');
  });

  it('displays user information in avatar dropdown', () => {
    renderWithRouter(<MainLayout />);

    expect(screen.getByTestId('avatar-dropdown')).toBeInTheDocument();
    expect(screen.getByTestId('current-user')).toHaveTextContent('Test User');
  });

  it('handles logout functionality', () => {
    renderWithRouter(<MainLayout />);

    fireEvent.click(screen.getByTestId('logout-btn'));
    expect(mockUseAuth.logout).toHaveBeenCalledTimes(1);
  });

  it('shows menu toggle button', () => {
    renderWithRouter(<MainLayout />);

    const menuButton = screen.getByTestId('icon-menu').parentElement;
    expect(menuButton).toBeInTheDocument();
  });

  it('handles menu toggle for desktop', () => {
    renderWithRouter(<MainLayout />);

    const menuButton = screen.getByTestId('icon-menu').parentElement;
    fireEvent.click(menuButton!);

    // Should toggle collapsed state
    expect(screen.getByTestId('nav')).toHaveAttribute('data-collapsed', 'true');
  });

  it('handles mobile responsive layout', async () => {
    // Mock mobile screen size
    window.innerWidth = 600;
    window.dispatchEvent(new Event('resize'));

    renderWithRouter(<MainLayout />);

    // 移动端：侧边栏收进抽屉，抽屉关闭时 nav 不渲染
    await waitFor(() => {
      expect(screen.queryByTestId('nav')).not.toBeInTheDocument();
      expect(screen.queryByTestId('side-sheet')).not.toBeInTheDocument();
    });
  });

  it('shows mobile drawer when menu is toggled on mobile', async () => {
    // Mock mobile screen size
    window.innerWidth = 600;
    window.dispatchEvent(new Event('resize'));

    renderWithRouter(<MainLayout />);

    const menuButton = screen.getByTestId('icon-menu').parentElement;
    fireEvent.click(menuButton!);

    await waitFor(() => {
      expect(screen.getByTestId('side-sheet')).toBeInTheDocument();
    });
  });

  it('closes mobile drawer when navigation item is selected', async () => {
    // Mock mobile screen size
    window.innerWidth = 600;
    window.dispatchEvent(new Event('resize'));

    renderWithRouter(<MainLayout />);

    // Open drawer
    const menuButton = screen.getByTestId('icon-menu').parentElement;
    fireEvent.click(menuButton!);

    await waitFor(() => {
      expect(screen.getByTestId('side-sheet')).toBeInTheDocument();
    });

    // Select navigation item
    fireEvent.click(screen.getByTestId('nav-item-/shortens'));

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/shortens');
    });
  });

  it('displays logo and title correctly', () => {
    renderWithRouter(<MainLayout />);

    expect(screen.getByTestId('title')).toHaveTextContent('Shortener');
  });

  it('shows collapsed logo when sidebar is collapsed', () => {
    renderWithRouter(<MainLayout />);

    // Toggle to collapsed state
    const menuButton = screen.getByTestId('icon-menu').parentElement;
    fireEvent.click(menuButton!);

    // Should show collapsed logo (S)
    expect(screen.getByText('S')).toBeInTheDocument();
  });

  it('handles window resize events', async () => {
    renderWithRouter(<MainLayout />);

    // Start with desktop size
    expect(window.innerWidth).toBe(1024);

    // Nav should be visible initially
    expect(screen.getByTestId('nav')).toBeInTheDocument();

    // Resize to mobile: nav moves into the closed drawer
    window.innerWidth = 600;
    window.dispatchEvent(new Event('resize'));

    await waitFor(() => {
      expect(screen.queryByTestId('nav')).not.toBeInTheDocument();
    });

    // Resize back to desktop: nav is rendered again
    window.innerWidth = 1024;
    window.dispatchEvent(new Event('resize'));

    await waitFor(() => {
      expect(screen.getByTestId('nav')).toBeInTheDocument();
    });
  });
});
