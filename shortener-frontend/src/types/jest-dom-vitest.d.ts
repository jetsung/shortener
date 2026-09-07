// vitest 5 的 Assertion 接口为双泛型参数（R, T），
// @testing-library/jest-dom 7.x 自带的 vitest 类型扩展仍为单参数，
// 泛型数量不一致导致 interface 合并静默失效，这里手动按 vitest 5 的签名适配。
// 运行时 matchers 由 src/test/setup.ts 中的 `@testing-library/jest-dom/vitest` 注册。
import type { TestingLibraryMatchers } from '@testing-library/jest-dom/matchers';

declare module 'vitest' {
  interface Assertion<
    R extends void | Promise<void> = void,
    T = unknown,
  > extends TestingLibraryMatchers<any, T> {}
  interface AsymmetricMatchersContaining extends TestingLibraryMatchers<any, any> {}
}
