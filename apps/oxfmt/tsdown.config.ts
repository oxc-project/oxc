import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['src-js/index.ts', 'src-js/cli.ts'],
  format: 'esm',
  platform: 'node',
  target: 'node20',
  dts: true,
  clean: true,
  outDir: 'dist',
  external: ['prettier'],
  shims: false,
});
