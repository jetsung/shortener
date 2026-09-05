import React, { forwardRef, useImperativeHandle } from 'react';
import { Form } from '@douyinfe/semi-ui-19';
import type { FormApi } from '@douyinfe/semi-ui-19/lib/es/form/interface';
import type { SemiFormProps, SemiFormRef } from './types';

export type { SemiFormProps, SemiFormRef } from './types';

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
    setValues: (values: Record<string, unknown>) => formApiRef.current?.setValues(values),
    getValues: () => formApiRef.current?.getValues() || {},
  }));

  const handleSubmit = async (values: Record<string, unknown>) => {
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
