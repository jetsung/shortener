import React, { forwardRef, useImperativeHandle } from 'react';
import { Form } from '@douyinfe/semi-ui-19';
import type { FormApi } from '@douyinfe/semi-ui-19/lib/es/form/interface';

export interface SemiFormProps {
  onFinish?: (values: any) => Promise<boolean> | boolean;
  onFinishFailed?: (errorInfo: any) => void;
  children?: React.ReactNode;
  labelPosition?: 'left' | 'top' | 'inset';
  labelWidth?: number | string;
  initValues?: any;
  [key: string]: any;
}

export interface SemiFormRef {
  submit: () => Promise<void>;
  validate: () => Promise<any>;
  reset: () => void;
  setValues: (values: any) => void;
  getValues: () => any;
}

/**
 * Semi Form 组件封装
 * 提供与 ProForm 相似的使用体验
 */
const SemiForm = forwardRef<SemiFormRef, SemiFormProps>((props, ref) => {
  const { onFinish, onFinishFailed, children, ...formProps } = props;
  const formApiRef = React.useRef<FormApi | null>(null);

  // 暴露表单方法到外部 ref
  useImperativeHandle(ref, () => ({
    submit: async () => {
      await formApiRef.current?.submitForm();
    },
    validate: async () => {
      return await formApiRef.current?.validate();
    },
    reset: () => formApiRef.current?.reset(),
    setValues: (values: any) => formApiRef.current?.setValues(values),
    getValues: () => formApiRef.current?.getValues() || {},
  }));

  const handleSubmit = async (values: unknown) => {
    try {
      if (onFinish) {
        return await onFinish(values);
      }
      return true;
    } catch (error) {
      if (onFinishFailed) {
        onFinishFailed(error);
      }
      return false;
    }
  };

  return (
    <Form
      onSubmit={handleSubmit}
      {...formProps}
      data-testid={formProps['data-testid'] || 'semi-form'}
      getFormApi={(api: FormApi) => {
        formApiRef.current = api;
      }}
    >
      {children}
    </Form>
  );
});

SemiForm.displayName = 'SemiForm';

export default SemiForm;
