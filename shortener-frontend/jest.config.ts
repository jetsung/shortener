export default {
  testEnvironment: 'jsdom',
  setupFiles: ['./src/test/setup.ts'],
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
  transform: {
    '^.+\\.(ts|tsx)$': 'ts-jest',
  },
  globals: {
    localStorage: null,
  },
};
