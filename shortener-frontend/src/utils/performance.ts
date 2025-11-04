import React from 'react';

/**
 * 创建懒加载组件的高阶函数
 * @param importFunc 动态导入函数
 * @param fallback 加载时的占位组件
 * @returns 懒加载组件
 */
export function createLazyComponent<T extends React.ComponentType<any>>(
  importFunc: () => Promise<{ default: T }>
) {
  return React.lazy(importFunc);
}

/**
 * 性能监控工具
 */
export class PerformanceMonitor {
  private static instance: PerformanceMonitor;
  private metrics: Map<string, number> = new Map();

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor();
    }
    return PerformanceMonitor.instance;
  }

  /**
   * 开始性能测量
   * @param name 测量名称
   */
  startMeasure(name: string): void {
    this.metrics.set(name, performance.now());
  }

  /**
   * 结束性能测量并返回耗时
   * @param name 测量名称
   * @returns 耗时（毫秒）
   */
  endMeasure(name: string): number {
    const startTime = this.metrics.get(name);
    if (startTime === undefined) {
      console.warn(`Performance measure "${name}" was not started`);
      return 0;
    }

    const duration = performance.now() - startTime;
    this.metrics.delete(name);

    // 在开发环境下输出性能信息
    if (import.meta.env.DEV) {
      console.log(`Performance: ${name} took ${duration.toFixed(2)}ms`);
    }

    return duration;
  }

  /**
   * 测量异步操作的性能
   * @param name 测量名称
   * @param asyncFn 异步函数
   * @returns 异步函数的结果
   */
  async measureAsync<T>(name: string, asyncFn: () => Promise<T>): Promise<T> {
    this.startMeasure(name);
    try {
      const result = await asyncFn();
      this.endMeasure(name);
      return result;
    } catch (error) {
      this.endMeasure(name);
      throw error;
    }
  }
}

/**
 * 预加载组件
 * @param importFunc 动态导入函数
 */
export function preloadComponent(importFunc: () => Promise<any>): void {
  // 在空闲时间预加载组件
  if ('requestIdleCallback' in window) {
    requestIdleCallback(() => {
      importFunc().catch(() => {
        // 预加载失败时静默处理
      });
    });
  } else {
    // 降级到 setTimeout
    setTimeout(() => {
      importFunc().catch(() => {
        // 预加载失败时静默处理
      });
    }, 100);
  }
}

/**
 * 检查是否支持 Web Vitals
 */
export function isWebVitalsSupported(): boolean {
  return 'PerformanceObserver' in window && 'PerformanceEntry' in window;
}

/**
 * 简单的 Web Vitals 监控
 */
export function initWebVitals(): void {
  if (!isWebVitalsSupported() || !import.meta.env.DEV) {
    return;
  }

  // 监控 LCP (Largest Contentful Paint)
  try {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      const lastEntry = entries[entries.length - 1];
      console.log('LCP:', lastEntry.startTime);
    });
    observer.observe({ entryTypes: ['largest-contentful-paint'] });
  } catch (e) {
    // 静默处理不支持的情况
  }

  // 监控 FID (First Input Delay)
  try {
    const observer = new PerformanceObserver((list) => {
      const entries = list.getEntries();
      entries.forEach((entry) => {
        console.log('FID:', (entry as any).processingStart - entry.startTime);
      });
    });
    observer.observe({ entryTypes: ['first-input'] });
  } catch (e) {
    // 静默处理不支持的情况
  }
}
