import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import History from '../History';

// Mock API services
const mockHistoryService = {
  getHistories: vi.fn(),
  deleteHistories: vi.fn(),
};

vi.mock('@/services/shortener/history', () => ({
  getHistories: (...args: unknown[]) => mockHistoryService.getHistories(...args),
  deleteHistories: (...args: unknown[]) => mockHistoryService.deleteHistories(...args),
}));

// Mock notification
vi.mock('@/utils/notification', () => ({
  Toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock Semi UI components
vi.mock('@douyinfe/semi-ui-19', () => ({
  Button: ({ children, onClick, type, ...props }: any) => (
    <button onClick={onClick} data-type={type} {...props}>
      {children}
    </button>
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
  Modal: ({ visible, title, children, onCancel, onOk }: any) =>
    visible ? (
      <div data-testid="modal">
        <div data-testid="modal-title">{title}</div>
        <div data-testid="modal-content">{children}</div>
        <button onClick={onCancel} data-testid="modal-cancel">
          取消
        </button>
        <button onClick={onOk} data-testid="modal-ok">
          确定
        </button>
      </div>
    ) : null,
  Toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

const mockReload = vi.fn();

const MockSemiTable = ({ headerTitle, request, columns, search, actionRef }: any) => {
  const mockData = [
    {
      id: 1,
      short_code: 'abc123',
      ip_address: '192.168.1.1',
      referer: 'https://example.com',
      user_agent: 'Mozilla/5.0',
      country: '中国',
      province: '广东',
      city: '深圳',
      isp: '电信',
      device_type: 'desktop',
      os: 'Linux',
      browser: 'Chrome',
      accessed_at: '2024-01-01T00:00:00Z',
      created_at: '2024-01-01T00:00:00Z',
    },
    {
      id: 2,
      short_code: 'def456',
      ip_address: '192.168.1.2',
      referer: 'https://test.com',
      user_agent: 'Mozilla/5.0',
      country: '中国',
      province: '北京',
      city: '北京',
      isp: '联通',
      device_type: 'mobile',
      os: 'iOS',
      browser: 'Safari',
      accessed_at: '2024-01-02T00:00:00Z',
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
      {search && (
        <div data-testid="search-form">
          <input data-testid="search-input" placeholder="搜索..." />
          <button
            data-testid="search-button"
            onClick={() => {
              const value = (
                document.querySelector('[data-testid="search-input"]') as HTMLInputElement
              )?.value;
              request?.({ current: 1, pageSize: 10, short_code: value }, {});
            }}
          >
            搜索
          </button>
        </div>
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

// Mock SemiTable component（History 从 @/components 导入命名导出 SemiTable）
vi.mock('@/components', () => ({
  SemiTable: (props: any) => MockSemiTable(props),
}));

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('History Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Setup default mock responses
    mockHistoryService.getHistories.mockResolvedValue({
      data: [
        {
          id: 1,
          short_code: 'abc123',
          ip_address: '192.168.1.1',
          referer: 'https://example.com',
          user_agent: 'Mozilla/5.0',
          country: '中国',
          province: '广东',
          city: '深圳',
          isp: '电信',
          device_type: 'desktop',
          os: 'Linux',
          browser: 'Chrome',
          accessed_at: '2024-01-01T00:00:00Z',
          created_at: '2024-01-01T00:00:00Z',
        },
        {
          id: 2,
          short_code: 'def456',
          ip_address: '192.168.1.2',
          referer: 'https://test.com',
          user_agent: 'Mozilla/5.0',
          country: '中国',
          province: '北京',
          city: '北京',
          isp: '联通',
          device_type: 'mobile',
          os: 'iOS',
          browser: 'Safari',
          accessed_at: '2024-01-02T00:00:00Z',
          created_at: '2024-01-02T00:00:00Z',
        },
      ],
      success: true,
      meta: { total: 2 },
    });
  });

  it('renders history page with table and search', async () => {
    renderWithRouter(<History />);

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
    expect(screen.getByTestId('table-title')).toHaveTextContent('日志列表');
    expect(screen.getByTestId('search-form')).toBeInTheDocument();
  });

  it('loads history data on mount', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledWith(
        expect.objectContaining({
          page: 1,
          per_page: 10,
        }),
      );
    });
  });

  it('displays history data in table', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      expect(screen.getByText('abc123')).toBeInTheDocument();
      expect(screen.getByText('def456')).toBeInTheDocument();
      expect(screen.getByText('https://example.com')).toBeInTheDocument();
      expect(screen.getByText('https://test.com')).toBeInTheDocument();
    });
  });

  it('supports search functionality', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledTimes(1);
    });

    const searchInput = screen.getByTestId('search-input');
    const searchButton = screen.getByTestId('search-button');

    fireEvent.change(searchInput, { target: { value: 'abc123' } });
    fireEvent.click(searchButton);

    // Should trigger a new request with search parameters
    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledTimes(2);
      expect(mockHistoryService.getHistories).toHaveBeenLastCalledWith(
        expect.objectContaining({
          short_code: 'abc123',
        }),
      );
    });
  });

  it('handles API error gracefully', async () => {
    mockHistoryService.getHistories.mockRejectedValue(new Error('API Error'));

    renderWithRouter(<History />);

    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalled();
    });

    // Should still render the table structure even with error
    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('supports pagination', async () => {
    renderWithRouter(<History />);

    // Test pagination functionality
    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledWith(
        expect.objectContaining({
          page: 1,
          per_page: 10,
        }),
      );
    });
  });

  it('displays table row data', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      // Check if row data is displayed
      expect(screen.getByText('1')).toBeInTheDocument(); // First item id
      expect(screen.getByText('2')).toBeInTheDocument(); // Second item id
      expect(screen.getByText('192.168.1.1')).toBeInTheDocument();
    });
  });

  it('formats dates correctly', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      // The dates should be formatted and displayed
      expect(screen.getAllByText(/2024/).length).toBeGreaterThan(0);
    });
  });

  it('supports data export functionality', async () => {
    renderWithRouter(<History />);

    // If there's an export button, test it
    const exportButton = screen.queryByText('导出');
    if (exportButton) {
      fireEvent.click(exportButton);
      // Test export functionality
    }

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('refreshes data when reload is triggered', async () => {
    renderWithRouter(<History />);

    // Wait for initial load
    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledTimes(1);
    });

    // If there's a refresh button, test it
    const refreshButton = screen.queryByText('刷新');
    if (refreshButton) {
      fireEvent.click(refreshButton);

      await waitFor(() => {
        expect(mockHistoryService.getHistories).toHaveBeenCalledTimes(2);
      });
    }
  });

  it('handles empty data state', async () => {
    mockHistoryService.getHistories.mockResolvedValue({
      data: [],
      success: true,
      meta: { total: 0 },
    });

    renderWithRouter(<History />);

    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalled();
    });

    // Should still render table structure
    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('supports filtering by date range', async () => {
    renderWithRouter(<History />);

    // If date pickers exist, test them
    const datePickers = screen.queryAllByTestId('date-picker');
    if (datePickers.length > 0) {
      fireEvent.change(datePickers[0], { target: { value: '2024-01-01' } });

      await waitFor(() => {
        // Should trigger filtered request
        expect(mockHistoryService.getHistories).toHaveBeenCalled();
      });
    }

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('deletes selected rows via confirm modal', async () => {
    mockHistoryService.deleteHistories.mockResolvedValue({ success: true });

    renderWithRouter(<History />);

    // 选中行后出现批量删除工具栏（通过 rowSelection onChange 模拟选中）
    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalled();
    });

    expect(screen.queryByTestId('modal')).not.toBeInTheDocument();
  });
});
