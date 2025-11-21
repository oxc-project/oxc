import { defineConfig, type UserConfig } from 'tsdown';

const commonConfig: UserConfig = {
  format: 'esm',
  platform: 'node',
  target: 'node20',
  outDir: 'dist',
  clean: true,
  unbundle: false,
  hash: false,
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
  define: { DEBUG: 'false' },
};

// Only generate `.d.ts` file for main export, not for CLI
const configs = defineConfig([
  {
    entry: 'src-js/cli.ts',
    ...commonConfig,
    dts: false,
  },
  {
    entry: 'src-js/index.ts',
    ...commonConfig,
    dts: { resolve: true },
    attw: true,
  },
]);

// Create separate debug build with debug assertions enabled
const debugConfigs = configs.map((config) => ({
  ...config,
  outDir: 'debug',
  define: { DEBUG: 'true' },
}));
configs.push(...debugConfigs);

export default configs;
