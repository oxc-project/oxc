import { defineConfig, type UserConfig } from 'tsdown';

const commonConfig: UserConfig = {
  format: ['esm'],
  platform: 'node',
  target: 'node20',
  outDir: 'dist',
  clean: true,
  unbundle: false,
  external: [
    // External native bindings
    './oxlint.*.node',
    'oxlint-*',
    // Files copied from `oxc-parser`.
    // Not bundled, to avoid needing sourcemaps when debugging.
    /^\.\.?\/.*\/dist\//,
  ],
  // At present only compress syntax.
  // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
  minify: { compress: true, mangle: false, codegen: { removeWhitespace: false } },
};

// Only generate `.d.ts` file for main export, not for CLI
export default defineConfig([
  {
    entry: {
      cli: 'src-js/cli.ts',
      plugins: 'src-js/plugins/index.ts',
    },
    ...commonConfig,
    dts: false,
  },
  {
    entry: 'src-js/index.ts',
    ...commonConfig,
    dts: true,
    attw: true,
  },
]);
