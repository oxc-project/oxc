import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['src-js/index.ts'],
  format: ['esm'],
  platform: 'node',
  target: 'node20',
  outDir: 'dist',
  clean: true,
  bundle: true,
  external: [
    // External the generated constants file - we'll copy it separately
    './generated/constants.cjs',
    // External native bindings
    './oxlint.*.node',
    'oxlint2-*',
    // External parser files - we'll copy them separately
    '../dist/parser/raw-transfer/lazy-common.cjs',
    '../dist/parser/generated/lazy/walk.cjs',
  ],
});
