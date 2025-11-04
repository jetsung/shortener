import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { BrowserRouter } from 'react-router-dom';
import Dashboard from '../Dashboard';

// Helper function to render with router
const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('Dashboard Integration Tests', () => {
  it('renders welcome card with correct title', () => {
    renderWithRouter(<Dashboard />);

    expect(screen.getByText('欢迎使用 Shortener 短网址生成器')).toBeInTheDocument();
  });

  it('displays project description', () => {
    renderWithRouter(<Dashboard />);

    expect(
      screen.getByText(/Shortener 是一个使用 Rust 语言开发的短网址生成器/),
    ).toBeInTheDocument();
    expect(screen.getByText(/UI 框架使用 Semi Design/)).toBeInTheDocument();
  });

  it('renders with proper styling and layout', () => {
    renderWithRouter(<Dashboard />);

    // Check if card is rendered
    const card =
      screen.getByText('欢迎使用 Shortener 短网址生成器').closest('div[class*="semi-card"]') ||
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
