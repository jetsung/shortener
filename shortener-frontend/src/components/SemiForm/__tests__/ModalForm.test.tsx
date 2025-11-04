import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SemiModalForm from '../ModalForm';

// Mock Semi UI components
vi.mock('@douyinfe/semi-ui', () => ({
  Modal: ({ title, visible, onOk, onCancel, children, okText, cancelText }: any) =>
    visible ? (
      <div data-testid="modal">
        <div data-testid="modal-title">{title}</div>
        <div data-testid="modal-content">{children}</div>
        <button onClick={onOk} data-testid="modal-ok">
          {okText || '确定'}
        </button>
        <button onClick={onCancel} data-testid="modal-cancel">
          {cancelText || '取消'}
        </button>
      </div>
    ) : null,
}));

// Mock SemiForm
vi.mock('../index', () => ({
  default: ({ children, onFinish, ...props }: any) => {
    const handleSubmit = (e: any) => {
      e.preventDefault();
      if (onFinish) {
        onFinish({});
      }
    };
    return (
      <form data-testid="semi-form" onSubmit={handleSubmit} {...props}>
        {children}
        <input name="test" data-testid="form-input" />
      </form>
    );
  },
}));

describe('SemiModalForm', () => {
  const defaultProps = {
    title: 'Test Modal',
    visible: true,
    onCancel: vi.fn(),
    onOk: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders modal when visible is true', () => {
    render(<SemiModalForm {...defaultProps} />);

    expect(screen.getByTestId('modal')).toBeInTheDocument();
    expect(screen.getByTestId('modal-title')).toHaveTextContent('Test Modal');
    expect(screen.getByTestId('semi-form')).toBeInTheDocument();
  });

  it('does not render modal when visible is false', () => {
    render(<SemiModalForm {...defaultProps} visible={false} />);

    expect(screen.queryByTestId('modal')).not.toBeInTheDocument();
  });

  it('calls onCancel when cancel button is clicked', () => {
    const onCancel = vi.fn();
    render(<SemiModalForm {...defaultProps} onCancel={onCancel} />);

    fireEvent.click(screen.getByTestId('modal-cancel'));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('calls onOk when ok button is clicked', async () => {
    const onOk = vi.fn();
    render(<SemiModalForm {...defaultProps} onOk={onOk} />);

    fireEvent.click(screen.getByTestId('modal-ok'));

    await waitFor(() => {
      expect(onOk).toHaveBeenCalledTimes(1);
    });
  });

  it('renders custom button text', () => {
    render(<SemiModalForm {...defaultProps} okText="保存" cancelText="关闭" />);

    expect(screen.getByTestId('modal-ok')).toHaveTextContent('保存');
    expect(screen.getByTestId('modal-cancel')).toHaveTextContent('关闭');
  });

  it('passes form props to SemiForm', () => {
    const onFinish = vi.fn();
    render(
      <SemiModalForm {...defaultProps} onFinish={onFinish} labelPosition="top" labelWidth={100} />,
    );

    const form = screen.getByTestId('semi-form');
    expect(form).toBeInTheDocument();
  });

  it('renders children inside the form', () => {
    render(
      <SemiModalForm {...defaultProps}>
        <div data-testid="custom-content">Custom Form Content</div>
      </SemiModalForm>,
    );

    expect(screen.getByTestId('custom-content')).toBeInTheDocument();
    expect(screen.getByTestId('custom-content')).toHaveTextContent('Custom Form Content');
  });

  it('handles modal props correctly', () => {
    render(
      <SemiModalForm
        {...defaultProps}
        width={600}
        modalProps={{ 'data-testid': 'custom-modal' }}
      />,
    );

    expect(screen.getByTestId('modal')).toBeInTheDocument();
  });

  it('handles async onOk correctly', async () => {
    const onOk = vi.fn().mockImplementation(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    render(<SemiModalForm {...defaultProps} onOk={onOk} />);

    fireEvent.click(screen.getByTestId('modal-ok'));

    await waitFor(() => {
      expect(onOk).toHaveBeenCalledTimes(1);
    });
  });

  it('handles onOk error gracefully', async () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
    const onOk = vi.fn().mockRejectedValue(new Error('Submit failed'));

    render(<SemiModalForm {...defaultProps} onOk={onOk} />);

    fireEvent.click(screen.getByTestId('modal-ok'));

    await waitFor(() => {
      expect(onOk).toHaveBeenCalledTimes(1);
      expect(consoleError).toHaveBeenCalledWith('Modal form submission failed:', expect.any(Error));
    });

    consoleError.mockRestore();
  });

  it('uses default width when not specified', () => {
    render(<SemiModalForm {...defaultProps} />);
    expect(screen.getByTestId('modal')).toBeInTheDocument();
  });

  it('applies default form props', () => {
    render(<SemiModalForm {...defaultProps} />);

    const form = screen.getByTestId('semi-form');
    expect(form).toBeInTheDocument();
  });
});
