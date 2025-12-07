import React, { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { Table, Button, Form, Card, Space, Toast, Typography, Spin } from '@douyinfe/semi-ui-19';
import { IconSearch, IconRefresh } from '@douyinfe/semi-icons';
import type { ColumnProps } from '@douyinfe/semi-ui-19/lib/es/table';

const { Text } = Typography;

export interface SemiTableColumn<T extends Record<string, any> = any>
  extends Omit<ColumnProps<T>, 'render'> {
  title: React.ReactNode;
  dataIndex: string;
  key?: string;
  width?: number;
  sorter?: boolean;
  hideInSearch?: boolean;
  hideInForm?: boolean;
  copyable?: boolean;
  valueType?: 'text' | 'textarea' | 'dateTime' | 'option';
  valueEnum?: Record<string, { text: string; status?: string }>;
  render?: (value: any, record: T, index: number) => React.ReactNode;
  renderText?: (text: any, record: T) => string;
}

export interface SemiTableActionRef {
  reload: () => void;
  reloadAndRest: () => void;
}

export interface SemiTableProps<T extends Record<string, any> = any, P = any> {
  headerTitle?: string;
  actionRef?: React.MutableRefObject<SemiTableActionRef | undefined>;
  rowKey: string;
  columns: SemiTableColumn<T>[];
  request?: (
    params: P & { current?: number; pageSize?: number },
    sorter: Record<string, any>,
    filter: Record<string, any>,
  ) => Promise<{
    data: T[];
    success: boolean;
    total: number;
  }>;
  search?: {
    labelWidth?: number;
  };
  toolBarRender?: () => React.ReactNode[];
  rowSelection?: {
    onChange: (selectedRowKeys: (string | number)[], selectedRows: T[]) => void;
  };
  columnsState?: {
    defaultValue?: Record<string, { show: boolean }>;
  };
  pagination?: boolean | object;
}

function SemiTable<T extends Record<string, any> = any, P = any>(props: SemiTableProps<T, P>) {
  const {
    headerTitle,
    actionRef,
    rowKey,
    columns,
    request,
    search,
    toolBarRender,
    rowSelection,
    columnsState,
    pagination = true,
  } = props;

  const [dataSource, setDataSource] = useState<T[]>([]);
  const [loading, setLoading] = useState(false);
  const [total, setTotal] = useState(0);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);
  const [sorter, setSorter] = useState<Record<string, any>>({});
  const [searchParams, setSearchParams] = useState<Record<string, any>>({});
  const [selectedRowKeys, setSelectedRowKeys] = useState<(string | number)[]>([]);
  const formRef = useRef<any>(null);

  // 处理列配置，支持隐藏列
  const processedColumns = columns
    .filter((col) => {
      const colKey = col.dataIndex || col.key;
      if (columnsState?.defaultValue && colKey) {
        return columnsState.defaultValue[colKey]?.show !== false;
      }
      return true;
    })
    .map((col) => {
      const processedCol: ColumnProps<T> = {
        ...col,
        key: col.key || col.dataIndex,
        sorter: col.sorter ? true : false,
      };

      // 处理复制功能
      if (col.copyable) {
        const originalRender = col.render;
        processedCol.render = (value: any, record: T, index: number) => {
          const displayValue = originalRender ? originalRender(value, record, index) : value;
          return displayValue;
        };
      }

      // 处理状态枚举
      if (col.valueEnum && !col.render) {
        processedCol.render = (value: any) => {
          const enumItem = col.valueEnum![value];
          if (enumItem) {
            return <Text type={enumItem.status as any}>{enumItem.text}</Text>;
          }
          return value;
        };
      }

      // 处理日期时间格式
      if (col.valueType === 'dateTime' && !col.render) {
        processedCol.render = (value: unknown) => {
          if (!value) return '-';
          if (typeof value === 'string' || typeof value === 'number') {
            return new Date(value).toLocaleString('zh-CN');
          }
          return '-';
        };
      }

      return processedCol;
    });

  // 加载数据
  const loadData = useCallback(async () => {
    if (!request) return;

    console.log('Loading data with:', { currentPage, pageSize, searchParams, sorter });
    setLoading(true);
    try {
      const params = {
        current: currentPage,
        pageSize,
        ...searchParams,
      } as P & { current?: number; pageSize?: number };

      console.log('Request params:', params);
      const result = await request(params, sorter, {});

      if (result.success) {
        setDataSource(result.data);
        setTotal(result.total);
        console.log('Data loaded:', {
          dataLength: result.data.length,
          total: result.total,
          currentPage,
          pageSize,
        });
      } else {
        Toast.error('数据加载失败');
      }
    } catch (error) {
      console.error('Table data loading error:', error);
      Toast.error('数据加载失败');
    } finally {
      setLoading(false);
    }
  }, [request, currentPage, pageSize, sorter, searchParams]);

  // 确保分页参数变化时重新加载数据
  useEffect(() => {
    if (request) {
      loadData();
    }
  }, [loadData]);

  // 暴露方法给 actionRef
  useEffect(() => {
    if (actionRef) {
      actionRef.current = {
        reload: loadData,
        reloadAndRest: () => {
          setCurrentPage(1);
          setSearchParams({});
          setSorter({});
          if (formRef.current) {
            formRef.current.reset();
          }
          // 重置后会触发 loadData
        },
      };
    }
  }, [actionRef, loadData]);

  // 处理搜索
  const handleSearch = (values: Record<string, unknown>) => {
    setSearchParams(values);
    setCurrentPage(1); // 搜索时重置到第一页
  };

  // 渲染搜索表单
  const renderSearchForm = () => {
    const searchColumns = columns.filter((col) => !col.hideInSearch);

    if (searchColumns.length === 0) return null;

    return (
      <Card style={{ marginBottom: 16 }}>
        <Form
          ref={formRef}
          layout="horizontal"
          onSubmit={handleSearch}
          labelWidth={search?.labelWidth || 120}
        >
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 16 }}>
            {searchColumns.map((col) => {
              // 如果有 valueEnum，渲染下拉选择框
              if (col.valueEnum) {
                const options = Object.entries(col.valueEnum).map(([key, value]) => ({
                  value: key,
                  label: value.text,
                }));

                return (
                  <Form.Select
                    key={col.dataIndex}
                    field={col.dataIndex}
                    label={col.title as string}
                    style={{ width: 200 }}
                    placeholder={`请选择${col.title}`}
                    optionList={options}
                  />
                );
              }

              // 默认渲染输入框
              return (
                <Form.Input
                  key={col.dataIndex}
                  field={col.dataIndex}
                  label={col.title as string}
                  style={{ width: 200 }}
                  placeholder={`请输入${col.title}`}
                />
              );
            })}
          </div>
          <div style={{ marginTop: 16 }}>
            <Space>
              <Button htmlType="submit" type="primary" icon={<IconSearch />}>
                查询
              </Button>
              <Button type="tertiary" icon={<IconRefresh />} htmlType="reset">
                重置
              </Button>
            </Space>
          </div>
        </Form>
      </Card>
    );
  };

  // 处理分页变化
  const handlePageChange = useCallback((page: number, size?: number) => {
    console.log('Page change from pagination:', { page, size });
    if (size !== undefined && size !== pageSize) {
      // 如果页面大小改变，先更新页面大小，这会触发重新加载
      setPageSize(size);
      setCurrentPage(1); // 改变页面大小时，重置到第一页
    } else {
      // 只改变页码
      setCurrentPage(page);
    }
  }, [pageSize]);

  // 处理页面大小变化
  const handlePageSizeChange = useCallback((size: number) => {
    console.log('Size change from pagination:', { size });
    setPageSize(size);
    setCurrentPage(1); // 改变页面大小时，重置到第一页
  }, []);

  const paginationConfig = useMemo(() => {
    if (pagination === false) {
      return false;
    }

    const baseConfig = {
      currentPage: currentPage,
      pageSize: pageSize,
      total: total,
      showSizeChanger: true,
      showQuickJumper: true,
      pageSizeOpts: [10, 20, 50, 100],
      onChange: handlePageChange,
      onPageSizeChange: handlePageSizeChange,
    };

    // 如果传入的是对象，合并配置，但确保关键属性不被覆盖
    if (typeof pagination === 'object') {
      const { onChange, onPageSizeChange, currentPage: _, ...restPagination } = pagination as any;
      return {
        ...restPagination,
        ...baseConfig,
      };
    }

    return baseConfig;
  }, [currentPage, pageSize, total, pagination, handlePageChange, handlePageSizeChange]);

  // 添加调试日志
  console.log('Pagination config:', paginationConfig);
  console.log('Current page in state:', currentPage);

  const rowSelectionConfig = rowSelection
    ? {
        selectedRowKeys,
        onChange: (selectedKeys?: (string | number)[], selectedRows?: T[]) => {
          if (selectedKeys) {
            setSelectedRowKeys(selectedKeys);
          }
          if (rowSelection?.onChange && selectedKeys && selectedRows) {
            rowSelection.onChange(selectedKeys, selectedRows);
          }
        },
      }
    : undefined;

  return (
    <div>
      {/* 标题和工具栏 */}
      {(headerTitle || toolBarRender) && (
        <Card style={{ marginBottom: 16 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            {headerTitle && <Typography.Title heading={4}>{headerTitle}</Typography.Title>}
            {toolBarRender && <Space>{toolBarRender()}</Space>}
          </div>
        </Card>
      )}

      {/* 搜索表单 */}
      {search && renderSearchForm()}

      {/* 表格 */}
      <div style={{ overflowX: 'auto' }}>
        <style>
          {`
            .compact-table .semi-table-tbody .semi-table-row {
              height: 40px !important;
            }
            .compact-table .semi-table-thead .semi-table-row {
              height: 40px !important;
            }
            .compact-table .semi-table-tbody .semi-table-row .semi-table-row-cell {
              padding: 8px 12px !important;
            }
            .compact-table .semi-table-thead .semi-table-row .semi-table-row-head {
              padding: 8px 12px !important;
            }
          `}
        </style>
        <Spin spinning={loading}>
          <div>
            {/* 调试信息 */}
            <div style={{ display: 'none' }}>
              CurrentPage: {currentPage}, PageSize: {pageSize}
            </div>
            <Table
              className="compact-table"
              dataSource={dataSource}
              columns={processedColumns}
              rowKey={rowKey}
              pagination={paginationConfig}
              rowSelection={rowSelectionConfig}
              scroll={{ x: 1200 }}
            />
          </div>
        </Spin>
      </div>
    </div>
  );
}

export default SemiTable;
