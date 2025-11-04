import React, { forwardRef, useImperativeHandle } from 'react';
import { Form } from '@douyinfe/semi-ui';
import type { FormApi } from '@douyinfe/semi-ui/lib/es/form/interface';

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
  const { onFinish, children, ...formProps } = props;
  let formApi: FormApi | null = null;

  // 暴露表单方法到外部 ref
  useImperativeHandle(ref, () => ({
    submit: () => formApi?.submitForm(),
    validate: () => formApi?.validate(),
    reset: () => formApi?.reset(),
    setValues: (values: any) => formApi?.setValues(values),
    getValues: () => formApi?.getValues() || {},
  }));

  const handleSubmit = async (values: any) => {
    if (onFinish) {
      return await onFinish(values);
    }
    return true;
  };

  return (
    <Form
      onSubmit={handleSubmit}
      {...formProps}
      getFormApi={(api: FormApi) => {
        formApi = api;
      }}
    >
      {children}
    </Form>
  );
});

SemiForm.displayName = 'SemiForm';

export default SemiForm;
