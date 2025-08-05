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
    // External native bindings
    './oxlint.*.node',
    'oxlint2-*',
    // External the generated constants file - we'll copy it separately
    './generated/constants.cjs',
    // These are generated (also used by oxc-parser, so we'll copy them separately)
    /..\/parser\/.*/,
  ],
  // At present only compress syntax.
  // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
  minify: { compress: true },
});
