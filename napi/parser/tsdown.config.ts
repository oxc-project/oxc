import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['./index.js'],
  external: [/..*\.node/],
});
