// Build configuration.
//
// 5 builds out of one source tree - the entry point, and the printer over the 2 build-time feature flags.
// The plugin order matters: `strip_ts` is a text transform, so it has to run before the plugins which parse.

import { defineConfig } from "tsdown";
import removeAssertsPlugin from "./tsdown_plugins/remove_asserts.ts";
import stripTsPlugin from "./tsdown_plugins/strip_ts.ts";
import unmapWritesPlugin from "./tsdown_plugins/unmap_writes.ts";
import constFunctionsPlugin from "./tsdown_plugins/const_functions.ts";

const isEnabled = (env: string | undefined) => env === "true" || env === "1";

// When run with `pnpm run build-dev`, generate a debug build with extra assertions.
// This is the build prepared by `pnpm run build-test` for conformance tests.
// It replaces the release build in `dist`, so rebuild with `pnpm run build` before benchmarking.
const DEBUG = isEnabled(process.env.DEBUG);

// When run with `pnpm run bench`, generate a build which honors the `skipSourcemapGeneration` option,
// so the benchmarks can measure the print pass without source map generation.
const BENCHMARKS = isEnabled(process.env.BENCHMARKS);

// Only remove assertions in release build. Debug builds keep `debugAssert` calls live.
const assertPlugins = DEBUG ? [] : [removeAssertsPlugin];

// Global constants defined at build time. See `src-js/globals.d.ts`.
// `DEBUG: false` lets the minifier remove the body of `debugAssert` and any other debug-only code.
const definedGlobals = {
  DEBUG: DEBUG ? "true" : "false",
  BENCHMARKS: BENCHMARKS ? "true" : "false",
};

// Base config.
// `platform: "node"` because the entry point loads the printer with `createRequire`.
const commonConfig = defineConfig({
  platform: "node",
  target: "node20",
  outDir: "dist",
  format: "esm",
  unbundle: false,
  hash: false,
  fixedExtension: false,
  // `scripts/build.ts` deletes `dist` before TSDown runs.
  // This allows generating all 5 builds into the same directory.
  clean: false,
  plugins: [...assertPlugins],
  inputOptions: {
    // For plugins
    experimental: { nativeMagicString: true },
  },
});

// Minification options.
// Release builds are minified in full - that is what is published, and it is half the size.
// Debug builds keep names and whitespace, so `dist` stays readable while working on the printer.
const minifyConfig = DEBUG
  ? {
      compress: { keepNames: { function: true, class: true } },
      mangle: false,
      codegen: { removeWhitespace: false },
    }
  : true;

// One printer build. The printer is built 4 times from `src-js/print/index.ts`,
// over 2 build-time feature flags (see `src-js/globals.d.ts`):
//
// - `SOURCEMAPS`: Source map support costs a little speed even when unused.
// - `TS`: TypeScript syntax support. JS-only builds lose all the TS field checks (via minifier dead-code removal)
//   and the TS switch arms + printer functions (via `strip_ts.ts`).
//
// `src-js/index.ts` loads whichever build the caller's options call for.
//
// In builds without source maps, nothing reads the `node` argument the mapping writes take,
// so `unmap_writes` rewrites every `writeWithMap` / `writeWithMapNoLast` call, and the imports
// which bring them in, into the plain `write` / `writeNoLast` they become without it.
const printerConfig = (name: string, { sourcemaps, ts }: { sourcemaps: boolean; ts: boolean }) => ({
  ...commonConfig,
  minify: minifyConfig,
  // Only the entry point's types are published
  dts: false,
  entry: { [name]: "src-js/print/index.ts" },
  define: {
    ...definedGlobals,
    SOURCEMAPS: sourcemaps ? "true" : "false",
    TS: ts ? "true" : "false",
  },
  plugins: [
    // `strip_ts` is a text transform, so must run before the AST-based plugins
    ...(ts ? [] : [stripTsPlugin()]),
    ...(sourcemaps ? [] : [unmapWritesPlugin]),
    constFunctionsPlugin,
    ...assertPlugins,
  ],
});

export default defineConfig([
  // Entry point
  {
    ...commonConfig,
    entry: { index: "src-js/index.ts" },
    minify: minifyConfig,
    dts: true,
    define: definedGlobals,
    deps: {
      // The printer builds are loaded at runtime, so must not be bundled in
      neverBundle: ["./print_js.js", "./print_js_maps.js", "./print_ts.js", "./print_ts_maps.js"],
    },
  },

  // Printers
  printerConfig("print_js", { sourcemaps: false, ts: false }),
  printerConfig("print_js_maps", { sourcemaps: true, ts: false }),
  printerConfig("print_ts", { sourcemaps: false, ts: true }),
  printerConfig("print_ts_maps", { sourcemaps: true, ts: true }),
]);
