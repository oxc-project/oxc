// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync } from "node:fs";
import { join } from "node:path";

const parserDirPath = join(import.meta.dirname, ".."),
  srcDirPath = join(parserDirPath, "src-js"),
  distDirPath = join(parserDirPath, "dist");

// Build with tsdown
console.log("Building with tsdown...");
execSync("pnpm tsdown", { stdio: "inherit", cwd: parserDirPath });

// Copy native `.node` files from `src-js` to `dist`
console.log("Copying .node, .wasm files to dist...");
for (const filename of readdirSync(srcDirPath)) {
  if (!filename.endsWith(".node") && !filename.endsWith(".wasm")) continue;
  const srcPath = join(srcDirPath, filename);
  copyFileSync(srcPath, join(distDirPath, filename));
}

// Copy type definitions (only the ones that are part of the public API)
console.log("Copying type definitions...");
copyFileSync(join(srcDirPath, "index.d.ts"), join(distDirPath, "index.d.ts"));
// visitor.d.ts is imported by index.d.ts
const visitDistDir = join(distDirPath, "generated", "visit");
mkdirSync(visitDistDir, { recursive: true });
copyFileSync(
  join(srcDirPath, "generated", "visit", "visitor.d.ts"),
  join(visitDistDir, "visitor.d.ts"),
);

// Copy browser/WASM related files that are not bundled
console.log("Copying browser/WASM files...");
const browserFiles = [
  "browser.js",
  "parser.wasi.cjs",
  "parser.wasi-browser.js",
  "wasi-worker.mjs",
  "wasi-worker-browser.mjs",
  "wasm.js",
  "webcontainer-fallback.cjs",
  "wrap.js",
];
for (const filename of browserFiles) {
  const srcPath = join(srcDirPath, filename);
  if (existsSync(srcPath)) {
    copyFileSync(srcPath, join(distDirPath, filename));
  }
}

console.log("Build complete!");
