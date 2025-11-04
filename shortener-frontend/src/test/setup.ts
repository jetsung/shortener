import '@testing-library/jest-dom'
import { vi } from 'vitest'

// 抑制ReactDOM.render警告
const originalError = console.error;
beforeAll(() => {
  console.error = (...args) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Warning: ReactDOM.render is no longer supported in React 18')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});

// Mock canvas for lottie-web
Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
  value: () => ({
    fillStyle: '',
    fillRect: () => {},
    clearRect: () => {},
    getImageData: () => ({ data: new Array(4) }),
    putImageData: () => {},
    createImageData: () => ({ data: new Array(4) }),
    setTransform: () => {},
    drawImage: () => {},
    save: () => {},
    restore: () => {},
    beginPath: () => {},
    moveTo: () => {},
    lineTo: () => {},
    closePath: () => {},
    stroke: () => {},
    fill: () => {},
    measureText: () => ({ width: 0 }),
    transform: () => {},
    translate: () => {},
    scale: () => {},
    rotate: () => {},
    arc: () => {},
    fillText: () => {},
  }),
});

// Mock lottie-web
vi.mock('lottie-web', () => ({
  default: {
    loadAnimation: () => ({
      play: () => {},
      pause: () => {},
      stop: () => {},
      destroy: () => {},
    }),
  },
}));
