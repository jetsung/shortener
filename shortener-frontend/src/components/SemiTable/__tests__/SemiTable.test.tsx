import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SemiTable from '../index';
import type { SemiTableColumn } from '../index';

// Mock Semi UI components
vi.mock('@douyinfe/semi-ui', () => ({
  Table: ({ dataSource, columns, loading, pagination, onChange }: any) => (
    <div data-testid="semi-table">
      {loading && <div data-testid="loading">Loading...</div>}
      <table>
        <thead>
          <tr>
            {columns.map((col: any) => (
              <th key={col.key || col.dataIndex}>{col.title}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {dataSource?.map((item: any, index: number) => (
            <tr key={item.id || index}>
              {columns.map((col: any) => (
                <td key={col.key || col.dataIndex}>
                  {col.render ? col.render(item[col.dataIndex], item, index) : item[col.dataIndex]}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {pagination && (
        <div data-testid="pagination">
          <button onClick={() => onChange?.({ sortField: 'name', sortOrder: 'ascend' })}>
            Sort
          </button>
        </div>
      )}
    </div>
  ),
  Button: ({ children, onClick, ...props }: any) => (
    <button onClick={onClick} {...props}>
      {children}
    </button>
  ),
  Form: Object.assign(
    ({ children, onSubmit, ...props }: any) => (
      <form onSubmit={onSubmit} {...props}>
        {children}
      </form>
    ),
    {
      Input: ({ field, label, placeholder, ...props }: any) => (
        <input
          name={field}
          placeholder={placeholder || `请输入${label}`}
          data-testid={`form-input-${field}`}
          {...props}
        />
      ),
      Select: ({ field, label, placeholder, optionList, ...props }: any) => (
        <select name={field} data-testid={`form-select-${field}`} {...props}>
          <option value="">{placeholder || `请选择${label}`}</option>
          {optionList?.map((opt: any) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      ),
    },
  ),
  Card: ({ children }: any) => <div data-testid="card">{children}</div>,
  Space: ({ children }: any) => <div data-testid="space">{children}</div>,
  Tooltip: ({ children }: any) => <div data-testid="tooltip">{children}</div>,
  Toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
  Typography: {
    Text: ({ children, type }: any) => <span data-type={type}>{children}</span>,
    Title: ({ children, heading }: any) => <h1 data-heading={heading}>{children}</h1>,
  },
  Spin: ({ children, spinning }: any) => (
    <div data-testid="spin" data-spinning={spinning}>
      {children}
    </div>
  ),
}));

vi.mock('@douyinfe/semi-icons', () => ({
  IconSearch: () => <span data-testid="icon-search">🔍</span>,
  IconRefresh: () => <span data-testid="icon-refresh">🔄</span>,
}));

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined),
  },
});

describe('SemiTable', () => {
  const mockColumns: SemiTableColumn[] = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      sorter: true,
    },
    {
      title: 'Age',
      dataIndex: 'age',
      key: 'age',
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      valueEnum: {
        active: { text: 'Active', status: 'success' },
        inactive: { text: 'Inactive', status: 'danger' },
      },
    },
    {
      title: 'URL',
      dataIndex: 'url',
      key: 'url',
      copyable: true,
    },
  ];

  const mockData = [
    { id: 1, name: 'John', age: 25, status: 'active', url: 'https://example.com' },
    { id: 2, name: 'Jane', age: 30, status: 'inactive', url: 'https://test.com' },
  ];

  const mockRequest = vi.fn().mockResolvedValue({
    data: mockData,
    success: true,
    total: 2,
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders table with basic props', () => {
    render(<SemiTable rowKey="id" columns={mockColumns} request={mockRequest} />);

    expect(screen.getByTestId('semi-table')).toBeInTheDocument();
    expect(screen.getByText('Name')).toBeInTheDocument();
    expect(screen.getByText('Age')).toBeInTheDocument();
  });

  it('displays header title when provided', () => {
    render(
      <SemiTable
        headerTitle="Test Table"
        rowKey="id"
        columns={mockColumns}
        request={mockRequest}
      />,
    );

    expect(screen.getByText('Test Table')).toBeInTheDocument();
  });

  it('calls request function on mount', async () => {
    render(<SemiTable rowKey="id" columns={mockColumns} request={mockRequest} />);

    await waitFor(() => {
      expect(mockRequest).toHaveBeenCalledWith({ current: 1, pageSize: 10 }, {}, {});
    });
  });

  it('handles loading state', () => {
    const slowRequest = vi.fn().mockImplementation(() => new Promise(() => {}));

    render(<SemiTable rowKey="id" columns={mockColumns} request={slowRequest} />);

    expect(screen.getByTestId('spin')).toHaveAttribute('data-spinning', 'true');
  });

  it('renders data correctly', async () => {
    render(<SemiTable rowKey="id" columns={mockColumns} request={mockRequest} />);

    await waitFor(() => {
      expect(screen.getByText('John')).toBeInTheDocument();
      expect(screen.getByText('Jane')).toBeInTheDocument();
      expect(screen.getByText('25')).toBeInTheDocument();
      expect(screen.getByText('30')).toBeInTheDocument();
    });
  });

  it('handles valueEnum rendering', async () => {
    render(<SemiTable rowKey="id" columns={mockColumns} request={mockRequest} />);

    await waitFor(() => {
      expect(screen.getByText('Active')).toBeInTheDocument();
      expect(screen.getByText('Inactive')).toBeInTheDocument();
    });
  });

  it('handles copyable columns', async () => {
    render(<SemiTable rowKey="id" columns={mockColumns} request={mockRequest} />);

    await waitFor(() => {
      // Copyable columns should display the URL values
      expect(screen.getByText('https://example.com')).toBeInTheDocument();
      expect(screen.getByText('https://test.com')).toBeInTheDocument();
    });
  });

  it('handles sorting', async () => {
    render(<SemiTable rowKey="id" columns={mockColumns} request={mockRequest} />);

    await waitFor(() => {
      const sortButton = screen.getByText('Sort');
      fireEvent.click(sortButton);
    });

    await waitFor(() => {
      // Request should be called with pagination params
      expect(mockRequest).toHaveBeenCalled();
      const lastCall = mockRequest.mock.calls[mockRequest.mock.calls.length - 1];
      expect(lastCall[0]).toMatchObject({ current: 1, pageSize: 10 });
    });
  });

  it('renders search form when search prop is provided', async () => {
    render(
      <SemiTable
        rowKey="id"
        columns={mockColumns}
        request={mockRequest}
        search={{ labelWidth: 120 }}
      />,
    );

    await waitFor(() => {
      expect(screen.getAllByTestId('card')).toHaveLength(1); // Search card
    });
  });

  it('handles actionRef methods', () => {
    const actionRef = { current: null } as React.MutableRefObject<any>;

    render(
      <SemiTable rowKey="id" columns={mockColumns} request={mockRequest} actionRef={actionRef} />,
    );

    expect(actionRef.current).toBeDefined();
    expect(actionRef.current?.reload).toBeInstanceOf(Function);
    expect(actionRef.current?.reloadAndRest).toBeInstanceOf(Function);
  });

  it('handles request failure', async () => {
    const failingRequest = vi.fn().mockRejectedValue(new Error('Request failed'));

    render(<SemiTable rowKey="id" columns={mockColumns} request={failingRequest} />);

    await waitFor(() => {
      expect(failingRequest).toHaveBeenCalled();
    });
  });

  it('handles pagination correctly', () => {
    render(
      <SemiTable
        rowKey="id"
        columns={mockColumns}
        request={mockRequest}
        pagination={{ pageSize: 20 }}
      />,
    );

    expect(screen.getByTestId('pagination')).toBeInTheDocument();
  });

  it('disables pagination when pagination is false', () => {
    render(
      <SemiTable rowKey="id" columns={mockColumns} request={mockRequest} pagination={false} />,
    );

    expect(screen.queryByTestId('pagination')).not.toBeInTheDocument();
  });
});
