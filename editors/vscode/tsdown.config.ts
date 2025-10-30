import { defineConfig } from 'tsdown';
import process from 'node:process';

export default defineConfig({
  entry: process.env.TEST === 'true' ? 'tests/**/*.ts' : { main: 'client/extension.ts' },
  external: ['vscode'],
  platform: 'node',
  target: 'node20',
  sourcemap: true,
  minify: true,
  outDir: 'out',
});
