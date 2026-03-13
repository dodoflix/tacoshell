import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'happy-dom',
    globals: true,
    // The web-app shell has no unit tests of its own yet (logic lives in
    // @tacoshell/ui). Allow the coverage step to pass with zero files so CI
    // is not blocked while the package is still being bootstrapped.
    passWithNoTests: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      reportsDirectory: './coverage',
    },
  },
})
