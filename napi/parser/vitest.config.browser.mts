import path from 'node:path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
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
  plugins: [
    {
      // patch cjs for browser mode since local file cannot be pre-bundled
      name: 'patch-cjs',
      transform(code, id, _options) {
        if (id.endsWith('napi/parser/wrap.js')) {
          return code.replace('module.exports.wrap =', 'export');
        }
      },
    },
  ],
});
