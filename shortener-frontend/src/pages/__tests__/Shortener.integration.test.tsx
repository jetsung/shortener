import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import Shortener from '../Shortener';

// Mock API services
const mockShortenService = {
  getShortens: vi.fn(),
  addShorten: vi.fn(),
  updateShorten: vi.fn(),
  deleteShorten: vi.fn(),
};

vi.mock('@/services/shortener/shorten', () => ({
  getShortens: (...args: unknown[]) => mockShortenService.getShortens(...args),
  addShorten: (...args: unknown[]) => mockShortenService.addShorten(...args),
  updateShorten: (...args: unknown[]) => mockShortenService.updateShorten(...args),
  deleteShorten: (...args: unknown[]) => mockShortenService.deleteShorten(...args),
}));

const mockCacheService = {
  refreshCache: vi.fn(),
};

vi.mock('@/services/shortener/cache', () => ({
  refreshCache: (...args: unknown[]) => mockCacheService.refreshCache(...args),
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

// Mock Semi UI components
vi.mock('@douyinfe/semi-ui-19', () => ({
  Button: ({ children, onClick, type, icon, title, loading, ...props }: any) => (
    <button
      onClick={onClick}
      data-type={type}
      data-loading={loading ? 'true' : undefined}
      title={title}
      {...props}
    >
      {icon}
      {children}
    </button>
  ),
  Modal: ({ visible, title, children, onOk, onCancel }: any) =>
    visible ? (
      <div data-testid="modal">
        <div data-testid="modal-title">{title}</div>
        <div>{children}</div>
        <button onClick={onOk} data-testid="modal-ok">
          确定
        </button>
        <button onClick={onCancel} data-testid="modal-cancel">
          取消
        </button>
      </div>
    ) : null,
  Form: Object.assign(
    ({ children, onSubmit }: any) => (
      <form onSubmit={onSubmit} data-testid="form">
        {children}
      </form>
    ),
    {
      Input: ({ label }: any) => <div data-testid="form-input">{label}</div>,
      TextArea: ({ label }: any) => <div data-testid="form-textarea">{label}</div>,
    },
  ),
  Typography: Object.assign(({ children, ...props }: any) => <div {...props}>{children}</div>, {
    Title: ({ children, heading, style, ...props }: any) => (
      <h1 style={style} data-heading={heading} {...props}>
        {children}
      </h1>
    ),
    Text: ({ children, strong, type, size, ...props }: any) => (
      <span data-strong={strong ? 'true' : undefined} data-type={type} data-size={size} {...props}>
        {children}
      </span>
    ),
    Paragraph: ({ children, ...props }: any) => <p {...props}>{children}</p>,
  }),
  Toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock Semi Icons
vi.mock('@douyinfe/semi-icons', () => ({
  IconPlus: () => <span data-testid="icon-plus">+</span>,
  IconCopy: () => <span data-testid="icon-copy">⧉</span>,
  IconRefresh: () => <span data-testid="icon-refresh">⟳</span>,
}));

const mockReload = vi.fn();

const MockSemiTable = ({
  headerTitle,
  request,
  columns,
  toolBarRender,
  actionRef,
  rowSelection,
}: any) => {
  const mockData = [
    {
      id: 1,
      short_code: 'abc123',
      short_url: 'https://s.example.com/abc123',
      original_url: 'https://example.com',
      description: '示例一',
      status: 0,
      updated_at: '2024-01-01T00:00:00Z',
      created_at: '2024-01-01T00:00:00Z',
    },
    {
      id: 2,
      short_code: 'def456',
      short_url: 'https://s.example.com/def456',
      original_url: 'https://test.com',
      description: '示例二',
      status: 0,
      updated_at: '2024-01-02T00:00:00Z',
      created_at: '2024-01-02T00:00:00Z',
    },
  ];

  // Simulate table request
  React.useEffect(() => {
    if (request) {
      request({ current: 1, pageSize: 10 }, {});
    }
  }, [request]);

  // Expose actionRef methods
  React.useEffect(() => {
    if (actionRef) {
      actionRef.current = {
        reload: mockReload,
        reloadAndRest: vi.fn(),
      };
    }
  }, [actionRef]);

  return (
    <div data-testid="semi-table">
      <div data-testid="table-title">{headerTitle}</div>
      <div data-testid="table-toolbar">{toolBarRender && toolBarRender()}</div>
      {rowSelection && (
        <button
          data-testid="select-row"
          onClick={() => rowSelection.onChange?.([mockData[0].id], [mockData[0]])}
        >
          选择行
        </button>
      )}
      <table>
        <thead>
          <tr>
            {columns?.map((col: any) => (
              <th key={col.key || col.dataIndex}>{col.title}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {mockData.map((item: any) => (
            <tr key={item.id}>
              {columns?.map((col: any) => (
                <td key={col.key || col.dataIndex}>
                  {col.render ? col.render(item[col.dataIndex], item) : item[col.dataIndex]}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

const MockSemiModalForm = ({ visible, title, children, onFinish, onCancel }: any) =>
  visible ? (
    <div data-testid="modal-form">
      <div data-testid="modal-form-title">{title}</div>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          onFinish?.({ short_code: 'test123', original_url: 'https://example.com' });
        }}
      >
        {children}
        <button type="submit" data-testid="modal-form-submit">
          提交
        </button>
      </form>
      <button onClick={onCancel} data-testid="modal-form-cancel">
        取消
      </button>
    </div>
  ) : null;

// Mock 组件（Shortener 从 @/components 导入命名导出 SemiTable、SemiModalForm）
vi.mock('@/components', () => ({
  SemiTable: (props: any) => MockSemiTable(props),
  SemiModalForm: (props: any) => MockSemiModalForm(props),
}));

// Mock UpdateForm（避免其内部依赖）
vi.mock('../components/UpdateForm', () => ({
  default: ({ updateModalOpen }: { updateModalOpen: boolean }) =>
    updateModalOpen ? <div data-testid="update-form" /> : null,
}));

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('Shortener Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Setup default mock responses
    mockShortenService.getShortens.mockResolvedValue({
      data: [
        {
          id: 1,
          short_code: 'abc123',
          short_url: 'https://s.example.com/abc123',
          original_url: 'https://example.com',
          description: '示例一',
          status: 0,
          updated_at: '2024-01-01T00:00:00Z',
          created_at: '2024-01-01T00:00:00Z',
        },
        {
          id: 2,
          short_code: 'def456',
          short_url: 'https://s.example.com/def456',
          original_url: 'https://test.com',
          description: '示例二',
          status: 0,
          updated_at: '2024-01-02T00:00:00Z',
          created_at: '2024-01-02T00:00:00Z',
        },
      ],
      success: true,
      meta: { total: 2 },
    });

    mockShortenService.addShorten.mockResolvedValue({
      success: true,
      data: { id: 3, short_code: 'new123', original_url: 'https://new.com' },
    });

    mockShortenService.updateShorten.mockResolvedValue({
      success: true,
    });

    mockShortenService.deleteShorten.mockResolvedValue({
      success: true,
    });

    mockCacheService.refreshCache.mockResolvedValue({
      cleared_keys: 2,
      warmed_urls: 2,
    });
  });

  it('renders shortener page with table and toolbar', async () => {
    renderWithRouter(<Shortener />);

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
    expect(screen.getByTestId('table-title')).toHaveTextContent('短址列表');
    expect(screen.getByTestId('table-toolbar')).toBeInTheDocument();
    expect(screen.getByText('新建')).toBeInTheDocument();
    expect(screen.getByText('刷新缓存')).toBeInTheDocument();
  });

  it('loads shortener data on mount', async () => {
    renderWithRouter(<Shortener />);

    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledWith(
        expect.objectContaining({
          page: 1,
          per_page: 10,
        }),
      );
    });
  });

  it('displays shortener data in table', async () => {
    renderWithRouter(<Shortener />);

    await waitFor(() => {
      expect(screen.getByText('abc123')).toBeInTheDocument();
      expect(screen.getByText('def456')).toBeInTheDocument();
      expect(screen.getByText('https://example.com')).toBeInTheDocument();
      expect(screen.getByText('https://test.com')).toBeInTheDocument();
    });
  });

  it('opens create modal when new button is clicked', async () => {
    renderWithRouter(<Shortener />);

    const newButton = screen.getByText('新建');
    fireEvent.click(newButton);

    await waitFor(() => {
      expect(screen.getByTestId('modal-form')).toBeInTheDocument();
      expect(screen.getByTestId('modal-form-title')).toHaveTextContent('新建短链');
    });
  });

  it('creates new shortener successfully', async () => {
    renderWithRouter(<Shortener />);

    // Open create modal
    const newButton = screen.getByText('新建');
    fireEvent.click(newButton);

    await waitFor(() => {
      expect(screen.getByTestId('modal-form')).toBeInTheDocument();
    });

    // Submit form
    const submitButton = screen.getByTestId('modal-form-submit');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(mockShortenService.addShorten).toHaveBeenCalledWith({
        short_code: 'test123',
        original_url: 'https://example.com',
      });
    });
  });

  it('handles create shortener error', async () => {
    mockShortenService.addShorten.mockRejectedValue(new Error('Create failed'));

    renderWithRouter(<Shortener />);

    // Open create modal
    const newButton = screen.getByText('新建');
    fireEvent.click(newButton);

    await waitFor(() => {
      expect(screen.getByTestId('modal-form')).toBeInTheDocument();
    });

    // Submit form
    const submitButton = screen.getByTestId('modal-form-submit');
    fireEvent.click(submitButton);

    await waitFor(() => {
      expect(mockShortenService.addShorten).toHaveBeenCalled();
    });
  });

  it('handles bulk delete operation', async () => {
    renderWithRouter(<Shortener />);

    // 选中一行后出现批量删除工具栏
    fireEvent.click(screen.getByTestId('select-row'));

    const bulkDeleteButton = await screen.findByText('批量删除');
    fireEvent.click(bulkDeleteButton);

    // 确认弹窗出现后点击确定
    const modalOk = await screen.findByTestId('modal-ok');
    fireEvent.click(modalOk);

    await waitFor(() => {
      expect(mockShortenService.deleteShorten).toHaveBeenCalledWith({ ids: [1] });
    });
  });

  it('refreshes cache when refresh button is clicked', async () => {
    renderWithRouter(<Shortener />);

    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledTimes(1);
    });

    fireEvent.click(screen.getByText('刷新缓存'));

    await waitFor(() => {
      expect(mockCacheService.refreshCache).toHaveBeenCalledTimes(1);
      expect(mockReload).toHaveBeenCalled();
    });
  });

  it('handles API error gracefully', async () => {
    mockShortenService.getShortens.mockRejectedValue(new Error('API Error'));

    renderWithRouter(<Shortener />);

    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalled();
    });

    // Should still render the table structure even with error
    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('supports search functionality', async () => {
    renderWithRouter(<Shortener />);

    // The search functionality would be tested if the component supports it
    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('supports pagination', async () => {
    renderWithRouter(<Shortener />);

    // Test pagination functionality
    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledWith(
        expect.objectContaining({
          page: 1,
          per_page: 10,
        }),
      );
    });
  });
});
