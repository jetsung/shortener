import type { ComponentProps, CSSProperties, ReactNode } from 'react';
import type { Modal } from '@douyinfe/semi-ui-19';

export interface SemiFormProps {
  onFinish?: (values: Record<string, unknown>) => Promise<boolean> | boolean;
  onFinishFailed?: (errorInfo: unknown) => void;
  children?: ReactNode;
  labelPosition?: 'left' | 'top' | 'inset';
  labelWidth?: number | string;
  initValues?: Record<string, unknown>;
  style?: CSSProperties;
  className?: string;
  'data-testid'?: string;
}

export interface SemiFormRef {
  submit: () => Promise<void>;
  validate: () => Promise<unknown>;
  reset: () => void;
  setValues: (values: Record<string, unknown>) => void;
  getValues: () => Record<string, unknown>;
}

export interface SemiModalFormProps extends SemiFormProps {
  title?: string;
  visible?: boolean;
  onCancel?: () => void;
  onOk?: () => Promise<void>;
  width?: number | string;
  okText?: string;
  cancelText?: string;
  /** 透传给 Modal 的其余属性 */
  modalProps?: Omit<ComponentProps<typeof Modal>, keyof SemiModalFormProps | 'children'> & {
    'data-testid'?: string;
  };
}
