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
    '@oxlint/*',
  ],
  fixedExtension: false,
  // At present only compress syntax.
  // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
  minify: {
    compress: { keepNames: { function: true, class: true } },
    mangle: false,
    codegen: { removeWhitespace: false },
  },
};

// Only generate `.d.ts` file for main export, not for CLI
export default defineConfig([
  {
    entry: 'src-js/cli.ts',
    ...commonConfig,
    dts: false,
  },
  {
    entry: 'src-js/index.ts',
    ...commonConfig,
    dts: {
      resolve: true,
    },
    attw: true,
  },
]);
