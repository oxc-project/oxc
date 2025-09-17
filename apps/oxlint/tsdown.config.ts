import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['src-js/cli.ts', 'src-js/plugins/index.ts'],
  format: ['esm'],
  platform: 'node',
  target: 'node20',
  outDir: 'dist',
  clean: true,
  bundle: true,
  external: [
    // External native bindings
    './oxlint.*.node',
    'oxlint-*',
    // Files copied from `oxc-parser`.
    // Not bundled, to avoid needing sourcemaps when debugging.
    /\/dist\//,
  ],
  // At present only compress syntax.
  // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
  minify: { compress: true, mangle: false, codegen: { removeWhitespace: false } },
});
