// oxlint-disable no-console

import { copyFileSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { rolldown } from "rolldown";

const parserDirPath = join(import.meta.dirname, "..");
const srcDirPath = join(parserDirPath, "src-js");
const distDirPath = join(parserDirPath, "dist");
const jsFilesToCopy = new Set(["bindings.js", "parser.wasi-browser.js"]);

const entries = {
  index: "src-js/index.js",
  wasm: "src-js/wasm.js",
  browser: "src-js/browser.js",
  wrap: "src-js/wrap.js",

  "raw-transfer/common": "src-js/raw-transfer/common.js",
  "raw-transfer/eager": "src-js/raw-transfer/eager.js",
  "raw-transfer/lazy": "src-js/raw-transfer/lazy.js",
  "raw-transfer/lazy-common": "src-js/raw-transfer/lazy-common.js",
  "raw-transfer/node-array": "src-js/raw-transfer/node-array.js",
  "raw-transfer/supported": "src-js/raw-transfer/supported.js",
  "raw-transfer/visitor": "src-js/raw-transfer/visitor.js",

  "visit/index": "src-js/visit/index.js",
  "visit/visitor": "src-js/visit/visitor.js",

  "generated/constants": "src-js/generated/constants.js",
  "generated/deserialize/js": "src-js/generated/deserialize/js.js",
  "generated/deserialize/js_parent": "src-js/generated/deserialize/js_parent.js",
  "generated/deserialize/js_range": "src-js/generated/deserialize/js_range.js",
  "generated/deserialize/js_range_parent": "src-js/generated/deserialize/js_range_parent.js",
  "generated/deserialize/ts": "src-js/generated/deserialize/ts.js",
  "generated/deserialize/ts_parent": "src-js/generated/deserialize/ts_parent.js",
  "generated/deserialize/ts_range": "src-js/generated/deserialize/ts_range.js",
  "generated/deserialize/ts_range_parent": "src-js/generated/deserialize/ts_range_parent.js",
  "generated/lazy/constructors": "src-js/generated/lazy/constructors.js",
  "generated/lazy/type_ids": "src-js/generated/lazy/type_ids.js",
  "generated/lazy/walk": "src-js/generated/lazy/walk.js",
  "generated/visit/keys": "src-js/generated/visit/keys.js",
  "generated/visit/type_ids": "src-js/generated/visit/type_ids.js",
  "generated/visit/walk": "src-js/generated/visit/walk.js",
};

console.log("Deleting `dist` directory...");
rmSync(distDirPath, { recursive: true, force: true });

console.log("Building with rolldown...");
const bundle = await rolldown({
  input: entries,
  platform: "node",
  external: (id) =>
    id === "./bindings.js" ||
    id === "../bindings.js" ||
    id === "@oxc-parser/binding-wasm32-wasi" ||
    id.startsWith("@oxc-parser/binding-"),
});

try {
  await bundle.write({
    dir: distDirPath,
    format: "esm",
    entryFileNames: "[name].js",
    chunkFileNames: "chunks/[name].js",
  });
} finally {
  await bundle.close();
}

console.log("Copying native, WASM, and declaration files...");
copyMatchingFiles(srcDirPath, distDirPath);

console.log("Build complete!");

/**
 * Copy files emitted by NAPI or consumed by declarations, preserving paths.
 * @param {string} srcDir - Source directory.
 * @param {string} destDir - Destination directory.
 * @returns {void}
 */
function copyMatchingFiles(srcDir, destDir) {
  for (const dirent of readdirSync(srcDir, { withFileTypes: true })) {
    const srcPath = join(srcDir, dirent.name);
    const destPath = join(destDir, dirent.name);

    if (dirent.isDirectory()) {
      copyMatchingFiles(srcPath, destPath);
      continue;
    }

    if (!shouldCopy(dirent.name)) continue;

    mkdirSync(dirname(destPath), { recursive: true });
    copyFileSync(srcPath, destPath);
    console.log(`- Copied ${relative(parserDirPath, destPath)}`);
  }
}

/**
 * @param {string} filename - File name.
 * @returns {boolean} Whether file should be copied to dist.
 */
function shouldCopy(filename) {
  return jsFilesToCopy.has(filename) || /\.(?:cjs|d\.ts|mjs|node|wasm)$/.test(filename);
}
