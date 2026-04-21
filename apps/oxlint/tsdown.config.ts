import { defineConfig } from "tsdown";
// oxlint-disable-next-line typescript/ban-ts-comment
// @ts-ignore - file is generated and not checked in to git
import ruleNames from "./src-js/generated/plugin-eslint/rule_names.ts";
import inlineSearchPlugin from "./tsdown_plugins/inline_search.ts";
import replaceGlobalsPlugin from "./tsdown_plugins/replace_globals.ts";
import replaceAssertsPlugin from "./tsdown_plugins/replace_asserts.ts";

const { env } = process;
const isEnabled = (env: string | undefined) => env === "true" || env === "1";

// When run with `CONFORMANCE=true pnpm run build-js`, generate a conformance build with alterations to behavior.
// Also enables debug assertions.
// This is the build used in conformance tests.
const CONFORMANCE = isEnabled(env.CONFORMANCE);

// When run with `DEBUG=true pnpm run build-js`, generate a debug build with extra assertions.
// This is the build used in tests.
const DEBUG = CONFORMANCE || isEnabled(env.DEBUG);

// Base config
const commonConfig = defineConfig({
  platform: "node",
  target: "node20",
  outDir: "dist",
  clean: true,
  unbundle: false,
  hash: false,
  fixedExtension: false,
  deps: {
    // tsdown warns about final bundled modules by `unbundle` + `deps.neverBundle`.
    // But we know what we are doing, just suppress the warnings.
    onlyBundle: false,
  },
});

// Minification options.
// At present only compress syntax.
// Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
const minifyConfig = {
  compress: { keepNames: { function: true, class: true } },
  mangle: false,
  codegen: { removeWhitespace: false },
};

// Defined globals.
// `DEBUG: false` allows minifier to remove debug assertions and debug-only code in release build.
const definedGlobals = {
  DEBUG: DEBUG ? "true" : "false",
  CONFORMANCE: CONFORMANCE ? "true" : "false",
};

// Base config for `@oxlint/plugins` package.
// "node12" target to match `engines` field of last ESLint 8 release (8.57.1).
const pluginsPkgConfig = defineConfig({
  ...commonConfig,
  entry: {
    index: "src-js/plugins.ts",
  },
  outDir: "dist-pkg-plugins",
  // `build.ts` deletes the directory before TSDown runs.
  // This allows generating the ESM and CommonJS builds in the same directory.
  clean: false,
  target: "node12",
  minify: minifyConfig,
  define: definedGlobals,
});

// Base config for `oxlint-plugin-eslint` package
const pluginEslintPkgConfig = defineConfig({
  ...commonConfig,
  outDir: "dist-pkg-plugin-eslint",
  minify: minifyConfig,
  // `build.ts` deletes the directory before TSDown runs.
  // This allows generating the ESM and CommonJS builds in the same directory.
  clean: false,
  dts: false,
});

// Build entries for `oxlint-plugin-eslint` rule files.
// Each rule is a separate CJS file, lazy-loaded on demand.
const pluginEslintRulesEntries: Record<string, string> = {};
for (const ruleName of ruleNames) {
  pluginEslintRulesEntries[`rules/${ruleName}`] =
    `src-js/generated/plugin-eslint/rules/${ruleName}.cjs`;
}

// Plugins.
// Only remove debug assertions in release build.
const plugins = [inlineSearchPlugin, replaceGlobalsPlugin];
if (!DEBUG) plugins.push(replaceAssertsPlugin);

// All build configs
export default defineConfig([
  // Main build
  {
    ...commonConfig,
    entry: ["src-js/cli.ts", "src-js/index.ts", "src-js/plugins-dev.ts"],
    format: "esm",
    deps: {
      ...commonConfig.deps,
      neverBundle: [
        // External native bindings
        "./oxlint.*.node",
        "@oxlint/*",
      ],
    },
    minify: minifyConfig,
    dts: true,
    attw: { profile: "esm-only" },
    define: definedGlobals,
    plugins,
    inputOptions: {
      // For `replaceAssertsPlugin` and `replaceGlobalsPlugin`
      experimental: { nativeMagicString: true },
    },
  },

  // `@oxlint/plugins` package.
  // Dual package - both ESM and CommonJS.
  {
    ...pluginsPkgConfig,
    format: "esm",
    dts: true,
  },
  {
    ...pluginsPkgConfig,
    format: "commonjs",
    dts: false,
  },

  // `oxlint-plugin-eslint` package
  {
    ...pluginEslintPkgConfig,
    entry: { index: "src-js/plugin-eslint/index.ts" },
    format: "esm",
    banner: {
      js: [
        "/**",
        " * ESLint's rules code copied from https://github.com/eslint/eslint",
        " *",
        " * License: MIT",
        " * https://github.com/eslint/eslint/blob/a0d1a3772679d3d74bb860fc65b5b58678acd452/LICENSE",
        " */",
      ].join("\n"),
    },
  },
  {
    ...pluginEslintPkgConfig,
    entry: pluginEslintRulesEntries,
    format: "commonjs",
    outputOptions: {
      chunkFileNames: "common/[name].cjs",
    },
  },
]);
