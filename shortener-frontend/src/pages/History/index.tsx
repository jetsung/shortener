import React, { useRef, useState } from 'react';
import { Button, Typography, Modal } from '@douyinfe/semi-ui-19';
import { Toast } from '@/utils/notification';
import { SemiTable } from '@/components';
import type { SemiTableActionRef, SemiTableColumn } from '@/components/SemiTable';
import { getHistories, deleteHistories } from '@/services/shortener/history';
import { useNavigate } from 'react-router-dom';

const { Text } = Typography;

interface HistoryResponse {
  id: number;
  short_code: string;
  ip_address: string;
  referer?: string;
  user_agent?: string;
  country?: string;
  province?: string;
  city?: string;
  isp?: string;
  device_type?: string;
  os?: string;
  browser?: string;
  accessed_at: string;
  created_at: string;
}

interface GetHistoriesParams {
  page?: number;
  page_size?: number;
  short_code?: string;
  sort_by?: string;
  order?: 'asc' | 'desc';
}

/**
 * 删除节点
 */
const handleRemove = async (selectedRows: HistoryResponse[]) => {
  Toast.info('正在删除');
  if (!selectedRows) return true;
  try {
    await deleteHistories({
      ids: selectedRows.map((row) => row.id).join(','),
    });
    Toast.success('删除成功，即将刷新');
    return true;
  } catch {
    Toast.error('删除失败，请重试');
    return false;
  }
};

const History: React.FC = () => {
  const actionRef = useRef<SemiTableActionRef>(undefined);
  const [selectedRowsState, setSelectedRows] = useState<HistoryResponse[]>([]);
  const navigate = useNavigate();

  const columns: SemiTableColumn<HistoryResponse>[] = [
    {
      title: 'ID',
      dataIndex: 'id',
      hideInSearch: true,
      width: 12,
      sorter: true,
    },
    {
      title: '短码',
      dataIndex: 'short_code',
      copyable: true,
      width: 150,
    },
    {
      title: '访问者 IP',
      dataIndex: 'ip_address',
      width: 120,
    },
    {
      title: '来源 URL',
      dataIndex: 'referer',
      hideInSearch: true,
    },
    {
      title: 'User-Agent',
      dataIndex: 'user_agent',
      hideInSearch: true,
      width: 320,
    },
    {
      title: '国家',
      dataIndex: 'country',
      hideInSearch: true,
      width: 80,
    },
    {
      title: '省份',
      dataIndex: 'province',
      hideInSearch: true,
      width: 80,
    },
    {
      title: '城市',
      dataIndex: 'city',
      hideInSearch: true,
      width: 80,
    },
    {
      title: '运营商',
      dataIndex: 'isp',
      hideInSearch: true,
      width: 85,
    },
    {
      title: '设备类型',
      dataIndex: 'device_type',
      hideInSearch: true,
    },
    {
      title: '操作系统',
      dataIndex: 'os',
      hideInSearch: true,
    },
    {
      title: '浏览器',
      dataIndex: 'browser',
      hideInSearch: true,
    },
    {
      title: '访问时间',
      dataIndex: 'accessed_at',
      valueType: 'dateTime',
      hideInSearch: true,
      width: 180,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      valueType: 'dateTime',
      hideInSearch: true,
      width: 180,
      sorter: true,
    },
  ];

  return (
    <div style={{ padding: 0 }}>
      <SemiTable<HistoryResponse, GetHistoriesParams>
        headerTitle="日志列表"
        actionRef={actionRef}
        rowKey="id"
        search={{
          labelWidth: 120,
        }}
        request={async (params, sorter) => {
          let data: HistoryResponse[] = [];
          let total = 0;
          let success = false;

          try {
            const { current: page, pageSize: page_size, ...rest } = params;
            const query: GetHistoriesParams = {
              page: page || 1,
              page_size: page_size || 10,
              ...rest,
            };
            const orderBy = Object.entries(sorter)[0];
            if (orderBy && orderBy.length === 2) {
              query.sort_by = orderBy[0];
              query.order = orderBy[1] === 'ascend' ? 'asc' : 'desc';
            }
            const res = await getHistories(query);
            data = (res.data || []) as HistoryResponse[];
            total = (res as { meta?: { total_items?: number } }).meta?.total_items || 0;
            success = true;
          } catch (error: unknown) {
            const err = error as { response?: { data?: { errinfo?: string }; status?: number } };
            const errinfo = err?.response?.data?.errinfo;
            Toast.error(errinfo ?? '数据获取失败');

            const status = err?.response?.status;
            if (status === 401) {
              navigate('/account/login');
            }
          }
          return {
            data: data,
            success: success,
            total: total,
          };
        }}
        columns={columns}
        columnsState={{
          // 配置默认隐藏的列
          defaultValue: {
            referer: { show: false },
            country: { show: false },
            region: { show: false },
            province: { show: false },
            city: { show: true },
            os: { show: false },
            device_type: { show: false },
            isp: { show: true },
          },
        }}
        rowSelection={{
          onChange: (_, selectedRows) => {
            setSelectedRows(selectedRows);
          },
        }}
      />

      {/* 批量删除工具栏 */}
      {selectedRowsState?.length > 0 && (
        <div
          style={{
            position: 'fixed',
            bottom: 0,
            left: 0,
            right: 0,
            background: 'var(--semi-color-bg-2)',
            padding: '16px 24px',
            borderTop: '1px solid var(--semi-color-border)',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            zIndex: 1000,
          }}
        >
          <Text>
            已选择 <Text strong>{selectedRowsState.length}</Text> 项
          </Text>
          <Button
            type="danger"
            onClick={() => {
              Modal.confirm({
                title: '确认删除',
                content: `确定要删除选中的 ${selectedRowsState.length} 条历史记录吗？此操作不可撤销。`,
                onOk: async () => {
                  await handleRemove(selectedRowsState);
                  setSelectedRows([]);
                  actionRef.current?.reloadAndRest?.();
                },
              });
            }}
          >
            批量删除
          </Button>
        </div>
      )}
    </div>
  );
};

export default History;
