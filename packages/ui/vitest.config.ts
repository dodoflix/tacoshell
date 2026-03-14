import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'json'],
      reportsDirectory: './coverage',
      thresholds: {
        lines: 80,
        branches: 80,
        functions: 80,
        statements: 80,
      },
      exclude: [
        'src/test-setup.ts',
        'src/**/*.stories.tsx',
        'src/types/**',
        'src/**/index.ts',
      ],
    },
  },
})
