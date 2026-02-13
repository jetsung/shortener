import React, { useEffect, useRef } from 'react';
import { Form } from '@douyinfe/semi-ui-19';
import { SemiModalForm } from '@/components';
import type { SemiFormRef } from '@/components/SemiForm/types';

export type FormValueType = {
  target?: string;
  template?: string;
  type?: string;
  time?: string;
  frequency?: string;
  short_code?: string;
  original_url?: string;
  description?: string;
};

export type UpdateFormProps = {
  onCancel: (flag?: boolean, formVals?: FormValueType) => void;
  onSubmit: (values: FormValueType) => Promise<boolean>;
  updateModalOpen: boolean;
  values: Partial<{
    id?: number;
    short_code?: string;
    short_url?: string;
    original_url?: string;
    description?: string;
    status?: number;
    created_at?: string;
    updated_at?: string;
  }>;
};

/**
 * 更新表单组件
 *
 * @param props 表单属性
 * @returns 返回模态表单组件
 */
const UpdateForm: React.FC<UpdateFormProps> = (props) => {
  const { onCancel, onSubmit, updateModalOpen, values } = props;
  const formRef = useRef<SemiFormRef>(null);

  useEffect(() => {
    if (updateModalOpen && values) {
      // 延迟设置表单值，确保表单完全渲染
      const timer = setTimeout(() => {
        if (formRef.current) {
          const formValues = {
            ...values,
            short_code: values.short_code || '',
            original_url: values.original_url || '',
            description: values.description || '',
          };

          formRef.current.setValues(formValues);
        }
      }, 100);

      return () => clearTimeout(timer);
    }
  }, [updateModalOpen, values]);

  return (
    <SemiModalForm
      ref={formRef}
      title="更新短链"
      visible={updateModalOpen}
      onFinish={async (formValues: FormValueType) => {
        try {
          const success = await onSubmit(formValues);
          if (success) {
            onCancel(false);
          }
          return success;
        } catch {
          return false;
        }
      }}
      onCancel={() => onCancel(false)}
      width={600}
      okText="确定"
      cancelText="取消"
    >
      <Form.Input
        field="short_code"
        label="短码"
        placeholder="短码"
        disabled
        rules={[{ required: true, message: '短码为必填项' }]}
      />
      <Form.Input
        field="original_url"
        label="源链接"
        placeholder="请输入源链接"
        rules={[
          { required: true, message: '源链接为必填项' },
          { type: 'url', message: '请输入有效的 URL' },
        ]}
      />
      <Form.TextArea field="description" label="链接描述" placeholder="链接描述" rows={3} />
    </SemiModalForm>
  );
};

export default UpdateForm;
