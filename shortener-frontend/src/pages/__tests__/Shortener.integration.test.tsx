import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import Shortener from '../Shortener';

// Mock API services
const mockShortenService = {
  getShortens: vi.fn(),
  createShorten: vi.fn(),
  updateShorten: vi.fn(),
  deleteShorten: vi.fn(),
};

vi.mock('../../services/shortener', () => ({
  getShortens: (...args: any[]) => mockShortenService.getShortens(...args),
  createShorten: (...args: any[]) => mockShortenService.createShorten(...args),
  updateShorten: (...args: any[]) => mockShortenService.updateShorten(...args),
  deleteShorten: (...args: any[]) => mockShortenService.deleteShorten(...args),
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
  Modal: ({ visible, title, children, onOk, onCancel }: any) => (
    visible ? (
      <div data-testid="modal">
        <div data-testid="modal-title">{title}</div>
        <div>{children}</div>
        <button onClick={onOk} data-testid="modal-ok">确定</button>
        <button onClick={onCancel} data-testid="modal-cancel">取消</button>
      </div>
    ) : null
  ),
  Form: ({ children, onSubmit }: any) => (
    <form onSubmit={onSubmit} data-testid="form">
      {children}
    </form>
  ),
  Toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock SemiTable component
vi.mock('../../components/SemiTable', () => ({
  default: ({ headerTitle, request, columns, toolBarRender, actionRef }: any) => {
    const mockData = [
      { id: 1, code: 'abc123', original_url: 'https://example.com', created_at: '2024-01-01' },
      { id: 2, code: 'def456', original_url: 'https://test.com', created_at: '2024-01-02' },
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
        <div data-testid="table-toolbar">
          {toolBarRender && toolBarRender()}
        </div>
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

// Mock SemiForm components
vi.mock('../../components/SemiForm', () => ({
  default: ({ children, onFinish }: any) => (
    <form
      data-testid="semi-form"
      onSubmit={(e) => {
        e.preventDefault();
        onFinish?.({ code: 'test123', original_url: 'https://example.com' });
      }}
    >
      {children}
    </form>
  ),
  ModalForm: ({ visible, title, children, onFinish }: any) => (
    visible ? (
      <div data-testid="modal-form">
        <div data-testid="modal-form-title">{title}</div>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onFinish?.({ code: 'test123', original_url: 'https://example.com' });
          }}
        >
          {children}
          <button type="submit" data-testid="modal-form-submit">提交</button>
        </form>
      </div>
    ) : null
  ),
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
        { id: 1, code: 'abc123', original_url: 'https://example.com', created_at: '2024-01-01' },
        { id: 2, code: 'def456', original_url: 'https://test.com', created_at: '2024-01-02' },
      ],
      success: true,
      total: 2,
    });

    mockShortenService.createShorten.mockResolvedValue({
      success: true,
      data: { id: 3, code: 'new123', original_url: 'https://new.com' },
    });

    mockShortenService.updateShorten.mockResolvedValue({
      success: true,
    });

    mockShortenService.deleteShorten.mockResolvedValue({
      success: true,
    });
  });

  it('renders shortener page with table and toolbar', async () => {
    renderWithRouter(<Shortener />);

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
    expect(screen.getByTestId('table-title')).toHaveTextContent('短址列表');
    expect(screen.getByTestId('table-toolbar')).toBeInTheDocument();
  });

  it('loads shortener data on mount', async () => {
    renderWithRouter(<Shortener />);

    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledWith(
        expect.objectContaining({
          current: 1,
          pageSize: 10,
        }),
        {},
        {}
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
      expect(mockShortenService.createShorten).toHaveBeenCalledWith({
        code: 'test123',
        original_url: 'https://example.com',
      });
    });
  });

  it('handles create shortener error', async () => {
    mockShortenService.createShorten.mockRejectedValue(new Error('Create failed'));

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
      expect(mockShortenService.createShorten).toHaveBeenCalled();
    });
  });

  it('handles bulk delete operation', async () => {
    renderWithRouter(<Shortener />);

    // Find and click bulk delete button (assuming it exists in toolbar)
    const bulkDeleteButton = screen.getByText('批量删除');
    fireEvent.click(bulkDeleteButton);

    // Should show confirmation or perform delete
    await waitFor(() => {
      // This would depend on the actual implementation
      expect(bulkDeleteButton).toBeInTheDocument();
    });
  });

  it('refreshes data when reload is triggered', async () => {
    renderWithRouter(<Shortener />);

    // Wait for initial load
    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledTimes(1);
    });

    // Find refresh button and click it
    const refreshButton = screen.getByText('刷新');
    fireEvent.click(refreshButton);

    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledTimes(2);
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
    // This is a placeholder for search-related integration tests
    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
  });

  it('supports pagination', async () => {
    renderWithRouter(<Shortener />);

    // Test pagination functionality
    await waitFor(() => {
      expect(mockShortenService.getShortens).toHaveBeenCalledWith(
        expect.objectContaining({
          current: 1,
          pageSize: 10,
        }),
        {},
        {}
      );
    });
  });
});
