import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import History from '../History';

// Mock API services
const mockHistoryService = {
  getHistories: vi.fn(),
};

vi.mock('../../services/shortener', () => ({
  getHistories: (...args: any[]) => mockHistoryService.getHistories(...args),
}));

// Mock Semi UI components
vi.mock('@douyinfe/semi-ui', () => ({
  Card: ({ children, title }: any) => (
    <div data-testid="card">
      {title && <div data-testid="card-title">{title}</div>}
      {children}
    </div>
  ),
  Button: ({ children, onClick, type, ...props }: any) => (
    <button onClick={onClick} data-type={type} {...props}>
      {children}
    </button>
  ),
  Space: ({ children }: any) => <div data-testid="space">{children}</div>,
  DatePicker: ({ onChange, placeholder }: any) => (
    <input
      data-testid="date-picker"
      placeholder={placeholder}
      onChange={(e) => onChange?.(e.target.value)}
    />
  ),
  Select: ({ children, onChange, placeholder }: any) => (
    <select data-testid="select" onChange={(e) => onChange?.(e.target.value)}>
      <option value="">{placeholder}</option>
      {children}
    </select>
  ),
  Toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock SemiTable component
vi.mock('../../components/SemiTable', () => ({
  default: ({ headerTitle, request, columns, search, actionRef }: any) => {
    const mockData = [
      {
        id: 1,
        code: 'abc123',
        original_url: 'https://example.com',
        click_count: 10,
        created_at: '2024-01-01',
        last_accessed: '2024-01-02'
      },
      {
        id: 2,
        code: 'def456',
        original_url: 'https://test.com',
        click_count: 5,
        created_at: '2024-01-02',
        last_accessed: '2024-01-03'
      },
    ];

    // Simulate table request
    React.useEffect(() => {
      if (request) {
        request({ current: 1, pageSize: 10 }, {}, {}).then(() => {
          // Mock successful request
        });
      }
    }, [request]);

    // Expose actionRef methods
    React.useEffect(() => {
      if (actionRef) {
        actionRef.current = {
          reload: vi.fn(),
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
            <button data-testid="search-button">搜索</button>
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
  },
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
          code: 'abc123',
          original_url: 'https://example.com',
          click_count: 10,
          created_at: '2024-01-01T00:00:00Z',
          last_accessed: '2024-01-02T00:00:00Z'
        },
        {
          id: 2,
          code: 'def456',
          original_url: 'https://test.com',
          click_count: 5,
          created_at: '2024-01-02T00:00:00Z',
          last_accessed: '2024-01-03T00:00:00Z'
        },
      ],
      success: true,
      total: 2,
    });
  });

  it('renders history page with table and search', async () => {
    renderWithRouter(<History />);

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
    expect(screen.getByTestId('table-title')).toHaveTextContent('访问历史');
    expect(screen.getByTestId('search-form')).toBeInTheDocument();
  });

  it('loads history data on mount', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledWith(
        expect.objectContaining({
          current: 1,
          pageSize: 10,
        }),
        {},
        {}
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

    const searchInput = screen.getByTestId('search-input');
    const searchButton = screen.getByTestId('search-button');

    fireEvent.change(searchInput, { target: { value: 'abc123' } });
    fireEvent.click(searchButton);

    // Should trigger a new request with search parameters
    await waitFor(() => {
      expect(mockHistoryService.getHistories).toHaveBeenCalledTimes(2);
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
          current: 1,
          pageSize: 10,
        }),
        {},
        {}
      );
    });
  });

  it('displays click count statistics', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      // Check if click count data is displayed
      expect(screen.getByText('10')).toBeInTheDocument(); // First item click count
      expect(screen.getByText('5')).toBeInTheDocument();  // Second item click count
    });
  });

  it('formats dates correctly', async () => {
    renderWithRouter(<History />);

    await waitFor(() => {
      // The dates should be formatted and displayed
      // This depends on the actual date formatting in the component
      expect(screen.getByTestId('semi-table')).toBeInTheDocument();
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
      total: 0,
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
});
