import { forwardRef, useRef, useImperativeHandle, useState } from 'react';
import { Modal } from '@douyinfe/semi-ui-19';
import SemiForm from './index';
import type { SemiFormRef, SemiModalFormProps } from './types';

/**
 * Semi Modal Form 组件
 * 结合 Modal 和 Form，提供与 ProForm ModalForm 相似的使用体验
 */
const SemiModalForm = forwardRef<SemiFormRef, SemiModalFormProps>((props, ref) => {
  const {
    title,
    visible,
    onCancel,
    onOk,
    width = 400,
    okText = '确定',
    cancelText = '取消',
    modalProps,
    children,
    ...formProps
  } = props;

  const internalFormRef = useRef<SemiFormRef>(null);
  const [loading, setLoading] = useState(false);

  // 暴露 SemiForm 的方法到外部 ref
  useImperativeHandle(ref, () => ({
    submit: () => internalFormRef.current?.submit() || Promise.resolve(),
    validate: () => internalFormRef.current?.validate() || Promise.resolve({}),
    reset: () => internalFormRef.current?.reset(),
    setValues: (values: any) => internalFormRef.current?.setValues(values),
    getValues: () => internalFormRef.current?.getValues() || {},
  }));

  const handleOk = async () => {
    try {
      setLoading(true);
      if (onOk) {
        await onOk();
      } else {
        await internalFormRef.current?.submit();
      }
    } catch (error) {
      console.error('Modal form submission failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    if (onCancel) {
      onCancel();
    }
  };

  return (
    <Modal
      title={title}
      visible={visible}
      onOk={handleOk}
      onCancel={handleCancel}
      width={width}
      okText={okText}
      cancelText={cancelText}
      confirmLoading={loading}
      {...modalProps}
    >
      <SemiForm ref={internalFormRef} labelPosition="left" labelWidth={80} {...formProps}>
        {children}
      </SemiForm>
    </Modal>
  );
});

SemiModalForm.displayName = 'SemiModalForm';

export default SemiModalForm;
