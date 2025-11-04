import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import Dashboard from '../Dashboard';

// Helper function to render with router
const renderWithRouter = (component: React.ReactElement) => {
  return render(
    <BrowserRouter>
      {component}
    </BrowserRouter>
  );
};

describe('Dashboard Integration Tests', () => {
  it('renders welcome card with correct title', () => {
    renderWithRouter(<Dashboard />);

    expect(screen.getByText('欢迎使用 Shortener 短网址生成器')).toBeInTheDocument();
  });

  it('displays project description with links', () => {
    renderWithRouter(<Dashboard />);

    expect(screen.getByText(/Shortener 是一个使用 Go 语言开发的短网址生成器/)).toBeInTheDocument();

    // Check for backend link
    const backendLink = screen.getByText('后端使用 Gin 框架');
    expect(backendLink).toBeInTheDocument();
    expect(backendLink.closest('a')).toHaveAttribute('href', 'https://git.jetsung.com/jetsung/shortener');
    expect(backendLink.closest('a')).toHaveAttribute('target', '_blank');

    // Check for frontend link
    const frontendLink = screen.getByText('前端使用 React 框架');
    expect(frontendLink).toBeInTheDocument();
    expect(frontendLink.closest('a')).toHaveAttribute('href', 'https://git.jetsung.com/idev/shortener-frontend');
    expect(frontendLink.closest('a')).toHaveAttribute('target', '_blank');
  });

  it('renders with proper styling and layout', () => {
    renderWithRouter(<Dashboard />);

    // Check if card is rendered
    const card = screen.getByText('欢迎使用 Shortener 短网址生成器').closest('div[class*="semi-card"]') ||
                 screen.getByText('欢迎使用 Shortener 短网址生成器').closest('div');
    expect(card).toBeInTheDocument();
  });

  it('has accessible content structure', () => {
    renderWithRouter(<Dashboard />);

    // Check for proper heading structure
    const title = screen.getByText('欢迎使用 Shortener 短网址生成器');
    expect(title.tagName).toBe('H3');

    // Check for description text
    const description = screen.getByText(/UI 框架使用 Semi Design/);
    expect(description).toBeInTheDocument();
  });

  it('renders without errors', () => {
    expect(() => {
      renderWithRouter(<Dashboard />);
    }).not.toThrow();
  });
});
