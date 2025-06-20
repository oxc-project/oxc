import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: [
    './src/lazy-common.js',
    './src/node-array.js',
    './src/generated/deserialize/js.js',
    './src/generated/deserialize/ts.js',
    './src/generated/deserialize/lazy.js',
  ],
});
