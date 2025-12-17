import { defineConfig } from "tsdown";

export default defineConfig({
  entry: [
    // Main entry point
    "src-js/index.js",
    // Raw transfer modules (lazy-loaded)
    "src-js/raw-transfer/common.js",
    "src-js/raw-transfer/eager.js",
    "src-js/raw-transfer/lazy.js",
    "src-js/raw-transfer/lazy-common.js",
    "src-js/raw-transfer/node-array.js",
    "src-js/raw-transfer/supported.js",
    "src-js/raw-transfer/visitor.js",
    // Visit modules (lazy-loaded)
    "src-js/visit/index.js",
    "src-js/visit/visitor.js",
    // Generated modules (will be converted to TS in future)
    "src-js/generated/constants.js",
    "src-js/generated/deserialize/js.js",
    "src-js/generated/deserialize/js_parent.js",
    "src-js/generated/deserialize/js_range.js",
    "src-js/generated/deserialize/js_range_parent.js",
    "src-js/generated/deserialize/ts.js",
    "src-js/generated/deserialize/ts_parent.js",
    "src-js/generated/deserialize/ts_range.js",
    "src-js/generated/deserialize/ts_range_parent.js",
    "src-js/generated/lazy/constructors.js",
    "src-js/generated/lazy/type_ids.js",
    "src-js/generated/lazy/walk.js",
    "src-js/generated/visit/keys.js",
    "src-js/generated/visit/type_ids.js",
    "src-js/generated/visit/walk.js",
  ],
  format: "esm",
  platform: "node",
  target: "node20",
  outDir: "dist",
  clean: true,
  unbundle: true,
  hash: false,
  fixedExtension: false,
  external: ["./oxc-parser.*.node", "@oxc-parser/*"],
  // At present only compress syntax.
  // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
  minify: {
    compress: { keepNames: { function: true, class: true } },
    mangle: false,
    codegen: { removeWhitespace: false },
  },
});
