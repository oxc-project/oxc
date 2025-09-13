import { defineConfig } from 'rolldown';

export default defineConfig({
  input: 'client/extension.ts',
  output: {
    file: 'out/main.js',
    sourcemap: true,
    format: 'cjs',
    banner: `"use strict";\n`,
    minify: true,
  },
  external: ['vscode'],
  platform: 'node',
  transform: {
    target: 'node16',
  },
});
