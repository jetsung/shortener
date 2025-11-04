import { useEffect, useRef, useCallback } from 'react';
import { PerformanceMonitor } from '../utils/performance';

/**
 * 性能监控 Hook
 * @param componentName 组件名称
 * @returns 性能监控方法
 */
export function usePerformance(componentName: string) {
  const performanceMonitor = useRef(PerformanceMonitor.getInstance());
  const renderCount = useRef(0);
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

  // 记录渲染次数
  useEffect(() => {
    renderCount.current += 1;
    if (import.meta.env.DEV && renderCount.current > 1) {
      console.log(`Component ${componentName} rendered ${renderCount.current} times`);
    }
  });

  // 测量异步操作性能
  const measureAsync = useCallback(
    async <T>(operationName: string, asyncFn: () => Promise<T>): Promise<T> => {
      return performanceMonitor.current.measureAsync(
        `${componentName}-${operationName}`,
        asyncFn
      );
    },
    [componentName]
  );

  // 开始测量
  const startMeasure = useCallback(
    (operationName: string) => {
      performanceMonitor.current.startMeasure(`${componentName}-${operationName}`);
    },
    [componentName]
  );

  // 结束测量
  const endMeasure = useCallback(
    (operationName: string) => {
      return performanceMonitor.current.endMeasure(`${componentName}-${operationName}`);
    },
    [componentName]
  );

  return {
    measureAsync,
    startMeasure,
    endMeasure,
    renderCount: renderCount.current,
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
          changedProps
        );
      } else if (renderCount.current > 1) {
        console.warn(
          `${componentName} re-rendered (${renderCount.current}) without props change - consider using React.memo`
        );
      }
    }

    prevProps.current = { ...props };
  });

  return renderCount.current;
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
