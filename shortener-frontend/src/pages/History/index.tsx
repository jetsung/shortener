import { useRef, useState } from 'react';
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
  per_page?: number;
  short_code?: string;
  ip_address?: string;
  sort_by?: string;
  order?: 'asc' | 'desc';
}

/**
 * 删除节点
 */
const handleRemove = async (selectedRows: HistoryResponse[]) => {
  if (!selectedRows) return false;
  try {
    await deleteHistories({
      ids: selectedRows.map((row) => row.id),
    });
    return true;
  } catch {
    return false;
  }
};

const History = () => {
  const actionRef = useRef<SemiTableActionRef>(undefined);
  const [selectedRowsState, setSelectedRows] = useState<HistoryResponse[]>([]);
  const [, _setDeleteSuccess] = useState(false);
  void _setDeleteSuccess; // 标记为已使用
  const [showConfirm, setShowConfirm] = useState(false);
  const navigate = useNavigate();

  const columns: SemiTableColumn<HistoryResponse>[] = [
    {
      title: 'ID',
      dataIndex: 'id',
      hideInSearch: true,
      sorter: true,
      width: 50,
      mobileWidth: 50,
    },
    {
      title: '短码',
      dataIndex: 'short_code',
      copyable: true,
      mobileWidth: 100,
    },
    {
      title: '访问者 IP',
      dataIndex: 'ip_address',
      mobileWidth: 120,
    },
    {
      title: '来源 URL',
      dataIndex: 'referer',
      hideInSearch: true,
      hideInMobile: true,
    },
    {
      title: 'User-Agent',
      dataIndex: 'user_agent',
      hideInSearch: true,
      hideInMobile: true,
    },
    {
      title: '国家',
      dataIndex: 'country',
      hideInSearch: true,
      hideInMobile: true,
      width: 80,
    },
    {
      title: '省份',
      dataIndex: 'province',
      hideInSearch: true,
      hideInMobile: true,
      width: 80,
    },
    {
      title: '城市',
      dataIndex: 'city',
      hideInSearch: true,
      hideInMobile: true,
      width: 80,
    },
    {
      title: '运营商',
      dataIndex: 'isp',
      hideInSearch: true,
      hideInMobile: true,
      width: 85,
    },
    {
      title: '设备类型',
      dataIndex: 'device_type',
      hideInSearch: true,
      hideInMobile: true,
    },
    {
      title: '操作系统',
      dataIndex: 'os',
      hideInSearch: true,
      hideInMobile: true,
    },
    {
      title: '浏览器',
      dataIndex: 'browser',
      hideInSearch: true,
      hideInMobile: true,
    },
    {
      title: '访问时间',
      dataIndex: 'accessed_at',
      valueType: 'dateTime',
      hideInSearch: true,
      hideInMobile: true,
      width: 160,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      valueType: 'dateTime',
      hideInSearch: true,
      hideInMobile: true,
      width: 160,
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
            const { current: page, pageSize: per_page, ...rest } = params;
            const query: GetHistoriesParams = {
              page: page || 1,
              per_page: per_page || 10,
              ...rest,
            };
            const orderBy = Object.entries(sorter)[0];
            if (orderBy && orderBy.length === 2) {
              query.sort_by = orderBy[0];
              query.order = orderBy[1] === 'ascend' ? 'asc' : 'desc';
            }
            const res = await getHistories(query);
            data = ((res as any).data || []) as HistoryResponse[];
            total = (res as any).meta?.total || 0;
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
        rowSelection={{
          onChange: (_, selectedRows) => {
            setSelectedRows(selectedRows);
          },
        }}
        expandable={{
          expandedRowRender: (record) => (
            <div style={{ padding: '12px', fontSize: '12px' }}>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>短码：</Text>
                <Text>{record.short_code}</Text>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>访问者 IP：</Text>
                <Text>{record.ip_address}</Text>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>来源 URL：</Text>
                <div style={{ wordBreak: 'break-all', whiteSpace: 'normal' }}>
                  {record.referer || '-'}
                </div>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>User-Agent：</Text>
                <div style={{ wordBreak: 'break-all', whiteSpace: 'normal' }}>
                  {record.user_agent || '-'}
                </div>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>位置：</Text>
                <Text>
                  {[record.country, record.province, record.city].filter(Boolean).join(' / ') ||
                    '-'}
                </Text>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>运营商：</Text>
                <Text>{record.isp || '-'}</Text>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>设备：</Text>
                <Text>{record.device_type || '-'}</Text>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>系统/浏览器：</Text>
                <Text>{[record.os, record.browser].filter(Boolean).join(' / ') || '-'}</Text>
              </div>
              <div style={{ marginBottom: '8px' }}>
                <Text strong>访问时间：</Text>
                <Text>{new Date(record.accessed_at).toLocaleString('zh-CN')}</Text>
              </div>
              <div>
                <Text strong>创建时间：</Text>
                <Text>{new Date(record.created_at).toLocaleString('zh-CN')}</Text>
              </div>
            </div>
          ),
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
          <Button type="danger" onClick={() => setShowConfirm(true)}>
            批量删除
          </Button>
        </div>
      )}

      <Modal
        visible={showConfirm}
        title="确认删除"
        onCancel={() => setShowConfirm(false)}
        onOk={async () => {
          const success = await handleRemove(selectedRowsState);
          setShowConfirm(false);
          if (success) {
            setSelectedRows([]);
            actionRef.current?.reload();
            Toast.success('删除成功');
          } else {
            Toast.error('删除失败，请重试');
          }
        }}
      >
        确定要删除选中的 {selectedRowsState.length} 条历史记录吗？此操作不可撤销。
      </Modal>
    </div>
  );
};

export default History;
