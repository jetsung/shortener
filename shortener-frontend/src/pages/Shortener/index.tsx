import React, { useRef, useState } from 'react';
import { Button, Form, Typography, Modal } from '@douyinfe/semi-ui';
import { Toast } from '@/utils/notification';
import { IconPlus, IconCopy } from '@douyinfe/semi-icons';
import { SemiTable, SemiModalForm } from '@/components';
import type { SemiTableActionRef, SemiTableColumn } from '@/components/SemiTable';
import {
  getShortens,
  addShorten,
  updateShorten,
  deleteShorten,
} from '@/services/shortener/shorten';
import type { FormValueType } from './components/UpdateForm';
import UpdateForm from './components/UpdateForm';
import { useNavigate } from 'react-router-dom';
import type { ShortenResponse, Shorten, GetShortensParams } from '@/types/api';

const { Text } = Typography;

const Shortener: React.FC = () => {
  const [createModalOpen, setCreateModalOpen] = useState<boolean>(false);
  const [updateModalOpen, setUpdateModalOpen] = useState<boolean>(false);
  const actionRef = useRef<SemiTableActionRef>(undefined);
  const [currentRow, setCurrentRow] = useState<ShortenResponse>();
  const [selectedRowsState, setSelectedRows] = useState<ShortenResponse[]>([]);
  const addFormRef = useRef<any>(null);
  const navigate = useNavigate();

  const copyToClipboard = (text: string, messageText: string) => {
    navigator.clipboard.writeText(text).then(() => {
      Toast.success(messageText);
    });
  };

  /**
   * 添加节点
   */
  const handleAdd = async (fields: Shorten) => {
    Toast.info('正在添加');
    try {
      await addShorten({
        ...fields,
      });
      Toast.update('添加成功', 'success');
      return true;
    } catch {
      Toast.update('添加失败，请重试！', 'error');
      return false;
    }
  };

  /**
   * 更新节点
   */
  const handleUpdate = async (fields: FormValueType) => {
    Toast.info('更新中');
    try {
      await updateShorten(
        {
          short_code: fields.short_code as string,
        },
        {
          original_url: fields.original_url as string,
          description: fields.description,
        },
      );
      Toast.update('更新成功', 'success');
      return true;
    } catch {
      Toast.update('更新失败，请重试', 'error');
      return false;
    }
  };

  /**
   * 删除节点
   */
  const handleRemove = async (selectedRows: ShortenResponse[]) => {
    Toast.info('正在删除');
    if (!selectedRows) return true;
    try {
      await deleteShorten({
        ids: selectedRows.map((row) => row.id).join(','),
      });
      Toast.update('删除成功，即将刷新', 'success');
      return true;
    } catch {
      Toast.update('删除失败，请重试', 'error');
      return false;
    }
  };

  const columns: SemiTableColumn<ShortenResponse>[] = [
    {
      title: 'ID',
      dataIndex: 'id',
      hideInSearch: true,
      width: 50,
      sorter: true,
    },
    {
      title: '短码',
      dataIndex: 'short_code',
      width: 120,
      render: (_, entity) => {
        return (
          <div style={{ display: 'flex', alignItems: 'center', gap: '0px' }}>
            <a
              href={entity.short_url}
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e: React.MouseEvent) => e.stopPropagation()}
              style={{ marginRight: '4px' }}
            >
              {entity.short_code}
            </a>
            <Button
              theme="borderless"
              size="small"
              icon={<IconCopy />}
              title="复制短码"
              onClick={() => copyToClipboard(entity.short_code as string, '短码复制成功')}
            />
            <Button
              theme="borderless"
              size="small"
              icon={<IconCopy />}
              title="复制短链"
              onClick={() => copyToClipboard(entity.short_url as string, '短链复制成功')}
            />
          </div>
        );
      },
    },
    {
      title: '源地址',
      dataIndex: 'original_url',
      copyable: true,
      width: 450,
    },
    {
      title: '描述',
      dataIndex: 'description',
      valueType: 'textarea',
      hideInSearch: true,
      width: 200,
      render: (text) => (
        <div
          style={{
            maxWidth: '180px',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
          title={text}
        >
          {text || '-'}
        </div>
      ),
    },
    {
      title: '状态',
      dataIndex: 'status',
      hideInForm: true,
      width: 80,
      valueEnum: {
        '': {
          text: '全部',
        },
        0: {
          text: '启用',
          status: 'success',
        },
        1: {
          text: '禁用',
          status: 'danger',
        },
        2: {
          text: '未知',
          status: 'warning',
        },
      },
    },
    {
      title: '最后更新时间',
      dataIndex: 'updated_at',
      valueType: 'dateTime',
      hideInSearch: true,
      width: 150,
      sorter: true,
    },
    {
      title: '创建时间',
      dataIndex: 'created_at',
      valueType: 'dateTime',
      hideInSearch: true,
      width: 150,
      sorter: true,
    },
    {
      title: '操作',
      dataIndex: 'option',
      valueType: 'option',
      hideInSearch: true,
      width: 80,
      render: (_, record) => [
        <Button
          key="update"
          theme="borderless"
          size="small"
          onClick={() => {
            setUpdateModalOpen(true);
            setCurrentRow(record);
          }}
        >
          更新
        </Button>,
      ],
    },
  ];

  return (
    <div style={{ padding: 0 }}>
      <SemiTable<ShortenResponse, GetShortensParams>
        headerTitle="短址列表"
        actionRef={actionRef}
        rowKey="id"
        search={{
          labelWidth: 120,
        }}
        toolBarRender={() => [
          <Button
            type="primary"
            key="primary"
            icon={<IconPlus />}
            onClick={() => {
              setCreateModalOpen(true);
            }}
          >
            新建
          </Button>,
        ]}
        request={async (params, sorter) => {
          let data: ShortenResponse[] = [];
          let total = 0;
          let success = false;

          try {
            const { current: page, pageSize: page_size, ...rest } = params;

            // 过滤掉空值参数，特别是 status 为空字符串时
            const filteredRest = Object.fromEntries(
              Object.entries(rest).filter(
                ([_, value]) => value !== '' && value !== null && value !== undefined,
              ),
            );

            const query: GetShortensParams = {
              page: page || 1,
              page_size: page_size || 10,
              ...filteredRest,
            };
            const orderBy = Object.entries(sorter)[0];
            if (orderBy && orderBy.length === 2) {
              query.sort_by = orderBy[0];
              query.order = orderBy[1] === 'ascend' ? 'asc' : 'desc';
            }
            const res = await getShortens(query);
            data = (res as any).data || [];
            total = (res as any).meta?.total || 0;
            success = true;
          } catch (error: unknown) {
            const errinfo = (error as any)?.response?.data?.errinfo;
            Toast.error(errinfo ?? '数据获取失败');

            const status = (error as any)?.response?.status;
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
              let modalInstance: any = null;
              modalInstance = Modal.confirm({
                title: '确认删除',
                content: `确定要删除选中的 ${selectedRowsState.length} 个短链吗？此操作不可撤销。`,
                onOk: async () => {
                  const success = await handleRemove(selectedRowsState);
                  if (success) {
                    setSelectedRows([]);
                    actionRef.current?.reloadAndRest?.();
                    // 手动关闭当前弹窗
                    modalInstance?.destroy?.();
                  }
                  return success;
                },
              });
            }}
          >
            批量删除
          </Button>
        </div>
      )}

      {/* 新建短链模态框 */}
      <SemiModalForm
        ref={addFormRef}
        title="新建短链"
        visible={createModalOpen}
        onFinish={async (values: unknown) => {
          const success = await handleAdd(values as Shorten);
          if (success) {
            setCreateModalOpen(false);
            addFormRef.current?.reset();
            actionRef.current?.reload();
          }
          return success;
        }}
        onCancel={() => setCreateModalOpen(false)}
        width={400}
        okText="确定"
        cancelText="取消"
      >
        <Form.Input field="short_code" label="短码" placeholder="请输入短码。可选" />
        <Form.Input
          field="original_url"
          label="源链接"
          placeholder="请输入源链接"
          rules={[
            { required: true, message: '源链接为必填项' },
            { type: 'url', message: '请输入有效的 URL' },
          ]}
        />
        <Form.TextArea field="description" label="描述" placeholder="链接描述" rows={3} />
      </SemiModalForm>

      {/* 更新短链模态框 */}
      <UpdateForm
        onSubmit={async (value) => {
          const success = await handleUpdate(value);
          if (success) {
            setUpdateModalOpen(false);
            setCurrentRow(undefined);
            actionRef.current?.reload();
          }
          return success;
        }}
        onCancel={(value?: boolean) => {
          setUpdateModalOpen(value ?? false);
        }}
        updateModalOpen={updateModalOpen}
        values={currentRow || {}}
      />
    </div>
  );
};

export default Shortener;
