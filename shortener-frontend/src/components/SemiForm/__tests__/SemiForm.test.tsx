import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useRef } from 'react';
import SemiForm from '../index';
import type { SemiFormRef } from '../index';

// Mock Semi UI Form component
vi.mock('@douyinfe/semi-ui', () => ({
  Form: vi.fn().mockImplementation(({ children, onSubmit, ...props }: any) => {
    const mockFormRef = {
      validate: vi.fn().mockResolvedValue({ test: 'value' }),
      reset: vi.fn(),
      setValues: vi.fn(),
      getValues: vi.fn().mockReturnValue({ test: 'value' }),
    };

    // Expose the mock ref for testing
    if (props.ref) {
      if (typeof props.ref === 'function') {
        props.ref(mockFormRef);
      } else if (props.ref && typeof props.ref === 'object') {
        props.ref.current = mockFormRef;
      }
    }

    const handleSubmit = (e: any) => {
      e.preventDefault();
      const formData = new FormData(e.target);
      const values = Object.fromEntries(formData.entries());
      onSubmit?.(values);
    };

    return (
      <form onSubmit={handleSubmit} data-testid="semi-form" {...props}>
        {children}
      </form>
    );
  }),
}));

// Test component that uses SemiForm
const TestFormComponent = ({ onFinish, onFinishFailed }: any) => {
  const formRef = useRef<SemiFormRef>(null);

  return (
    <div>
      <SemiForm
        ref={formRef}
        onFinish={onFinish}
        onFinishFailed={onFinishFailed}
      >
        <input name="username" data-testid="username-input" />
        <input name="email" data-testid="email-input" />
      </SemiForm>
      <button
        onClick={() => formRef.current?.submit()}
        data-testid="external-submit"
      >
        External Submit
      </button>
      <button
        onClick={() => formRef.current?.reset()}
        data-testid="external-reset"
      >
        External Reset
      </button>
      <button
        onClick={() => formRef.current?.setValues({ username: 'test', email: 'test@example.com' })}
        data-testid="external-set-values"
      >
        Set Values
      </button>
    </div>
  );
};

describe('SemiForm', () => {
  const mockOnFinish = vi.fn();
  const mockOnFinishFailed = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders form correctly', () => {
    render(
      <SemiForm onFinish={mockOnFinish}>
        <input data-testid="test-input" />
      </SemiForm>
    );

    expect(screen.getByTestId('semi-form')).toBeInTheDocument();
    expect(screen.getByTestId('test-input')).toBeInTheDocument();
  });

  it('calls onFinish when form is submitted', async () => {
    mockOnFinish.mockResolvedValue(true);

    render(
      <SemiForm onFinish={mockOnFinish}>
        <input name="test" defaultValue="value" />
        <button type="submit" data-testid="submit-btn">Submit</button>
      </SemiForm>
    );

    fireEvent.click(screen.getByTestId('submit-btn'));

    await waitFor(() => {
      expect(mockOnFinish).toHaveBeenCalledWith({ test: 'value' });
    });
  });

  it('calls onFinishFailed when onFinish throws error', async () => {
    const error = new Error('Submit failed');
    mockOnFinish.mockRejectedValue(error);

    render(
      <SemiForm onFinish={mockOnFinish} onFinishFailed={mockOnFinishFailed}>
        <input name="test" defaultValue="value" />
        <button type="submit" data-testid="submit-btn">Submit</button>
      </SemiForm>
    );

    fireEvent.click(screen.getByTestId('submit-btn'));

    await waitFor(() => {
      expect(mockOnFinishFailed).toHaveBeenCalledWith(error);
    });
  });

  it('exposes correct ref methods', () => {
    const TestComponent = () => {
      const formRef = useRef<SemiFormRef>(null);

      return (
        <div>
          <SemiForm ref={formRef} onFinish={mockOnFinish}>
            <input data-testid="test-input" />
          </SemiForm>
          <button
            onClick={() => {
              expect(formRef.current?.submit).toBeInstanceOf(Function);
              expect(formRef.current?.validate).toBeInstanceOf(Function);
              expect(formRef.current?.reset).toBeInstanceOf(Function);
              expect(formRef.current?.setValues).toBeInstanceOf(Function);
              expect(formRef.current?.getValues).toBeInstanceOf(Function);
            }}
            data-testid="check-methods"
          >
            Check Methods
          </button>
        </div>
      );
    };

    render(<TestComponent />);
    fireEvent.click(screen.getByTestId('check-methods'));
  });

  it('handles external submit via ref', async () => {
    mockOnFinish.mockResolvedValue(true);

    render(<TestFormComponent onFinish={mockOnFinish} />);

    // Fill form
    fireEvent.change(screen.getByTestId('username-input'), {
      target: { value: 'testuser' }
    });
    fireEvent.change(screen.getByTestId('email-input'), {
      target: { value: 'test@example.com' }
    });

    // Submit via external button
    fireEvent.click(screen.getByTestId('external-submit'));

    await waitFor(() => {
      expect(mockOnFinish).toHaveBeenCalled();
    });
  });

  it('passes through additional props to Form component', () => {
    render(
      <SemiForm
        onFinish={mockOnFinish}
        labelPosition="top"
        labelWidth={100}
        data-testid="custom-form"
      >
        <input data-testid="test-input" />
      </SemiForm>
    );

    const form = screen.getByTestId('semi-form');
    expect(form).toHaveAttribute('data-testid', 'custom-form');
  });

  it('handles async onFinish correctly', async () => {
    const asyncOnFinish = vi.fn().mockImplementation(async (_values) => {
      await new Promise(resolve => setTimeout(resolve, 100));
      return true;
    });

    render(
      <SemiForm onFinish={asyncOnFinish}>
        <input name="test" defaultValue="value" />
        <button type="submit" data-testid="submit-btn">Submit</button>
      </SemiForm>
    );

    fireEvent.click(screen.getByTestId('submit-btn'));

    await waitFor(() => {
      expect(asyncOnFinish).toHaveBeenCalledWith({ test: 'value' });
    });
  });

  it('handles form without onFinish callback', () => {
    render(
      <SemiForm>
        <input name="test" defaultValue="value" />
        <button type="submit" data-testid="submit-btn">Submit</button>
      </SemiForm>
    );

    // Should not throw error when submitting without onFinish
    expect(() => {
      fireEvent.click(screen.getByTestId('submit-btn'));
    }).not.toThrow();
  });
});
