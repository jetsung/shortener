import { useEffect, useRef, useCallback } from 'react';
import { PerformanceMonitor } from '../utils/performance';

/**
 * 性能监控 Hook
 * @param componentName 组件名称
 * @returns 性能监控方法
 */
export function usePerformance(componentName: string) {
  const performanceMonitor = useRef(PerformanceMonitor.getInstance());
  const mountTime = useRef<number | undefined>(undefined);

  // 组件挂载时开始计时
  useEffect(() => {
    mountTime.current = performance.now();
    performanceMonitor.current.startMeasure(`${componentName}-mount`);

    return () => {
      // 组件卸载时结束计时
      if (mountTime.current) {
        const mountDuration = performance.now() - mountTime.current;
        if (import.meta.env.DEV) {
          console.log(`Component ${componentName} was mounted for ${mountDuration.toFixed(2)}ms`);
        }
      }
      performanceMonitor.current.endMeasure(`${componentName}-mount`);
    };
  }, [componentName]);

  // 测量异步操作性能
  const measureAsync = useCallback(
    async <T>(operationName: string, asyncFn: () => Promise<T>): Promise<T> => {
      return performanceMonitor.current.measureAsync(`${componentName}-${operationName}`, asyncFn);
    },
    [componentName],
  );

  // 开始测量
  const startMeasure = useCallback(
    (operationName: string) => {
      performanceMonitor.current.startMeasure(`${componentName}-${operationName}`);
    },
    [componentName],
  );

  // 结束测量
  const endMeasure = useCallback(
    (operationName: string) => {
      return performanceMonitor.current.endMeasure(`${componentName}-${operationName}`);
    },
    [componentName],
  );

  return {
    measureAsync,
    startMeasure,
    endMeasure,
  };
}

/**
 * 组件渲染优化 Hook
 * 用于检测不必要的重新渲染
 */
export function useRenderOptimization(componentName: string, props: Record<string, any>) {
  const prevProps = useRef<Record<string, any> | undefined>(undefined);
  const renderCount = useRef(0);

  useEffect(() => {
    // 在 effect 中更新计数和检查 props 变化
    renderCount.current += 1;

    if (import.meta.env.DEV && prevProps.current) {
      const changedProps: string[] = [];

      // 检查哪些 props 发生了变化
      Object.keys(props).forEach((key) => {
        if (props[key] !== prevProps.current![key]) {
          changedProps.push(key);
        }
      });

      if (changedProps.length > 0) {
        console.log(
          `${componentName} re-rendered (${renderCount.current}) due to props change:`,
          changedProps,
        );
      } else if (renderCount.current > 1 && renderCount.current % 100 === 0) {
        console.warn(
          `${componentName} re-rendered (${renderCount.current}) without props change - possible infinite loop!`,
        );
      }
    }

    prevProps.current = { ...props };
  });

  // 返回一个固定值，避免在渲染时访问 ref
  return 0;
}

/**
 * 内存使用监控 Hook
 */
export function useMemoryMonitor(componentName: string) {
  const checkMemory = useCallback(() => {
    if ('memory' in performance && import.meta.env.DEV) {
      const memory = (performance as any).memory;
      console.log(`${componentName} Memory Usage:`, {
        used: `${(memory.usedJSHeapSize / 1024 / 1024).toFixed(2)} MB`,
        total: `${(memory.totalJSHeapSize / 1024 / 1024).toFixed(2)} MB`,
        limit: `${(memory.jsHeapSizeLimit / 1024 / 1024).toFixed(2)} MB`,
      });
    }
  }, [componentName]);

  useEffect(() => {
    // 组件挂载时检查内存
    checkMemory();
  }, [checkMemory]);

  return { checkMemory };
}
