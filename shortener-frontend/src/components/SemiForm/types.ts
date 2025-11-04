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

export interface SemiModalFormProps extends SemiFormProps {
  title?: string;
  visible?: boolean;
  onCancel?: () => void;
  onOk?: () => Promise<void>;
  width?: number | string;
  okText?: string;
  cancelText?: string;
  modalProps?: any;
}
