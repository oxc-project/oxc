import path from 'node:path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  define: {
    'process.env.NODE_DEBUG_NATIVE': '"wasi"',
  },
  test: {
    dir: 'test-browser',
    browser: {
      enabled: true,
      provider: 'playwright',
      instances: [
        {
          browser: 'chromium',
        },
      ],
    },
  },
  resolve: {
    alias: {
      '@oxc-parser/binding-wasm32-wasi': path.resolve('npm-dir/wasm32-wasi'),
    },
  },
  server: {
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
  },
});
