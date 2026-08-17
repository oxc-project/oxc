// oxlint-disable no-console
// Build the JS corpus for the JS-plugin worker benchmark.
//
// Sources are real library code from the repo's own `node_modules/.pnpm` store, not generated
// files. Selection is deterministic (sorted paths), so the same arguments always produce the same
// tree. Bundled/minified files are skipped: one 200 KB line is not what a plugin walks in practice.
//
// Usage: node make-corpus.mjs <repo-root> <out-dir> <copies>

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const [repoRoot, outDir, copiesArg] = process.argv.slice(2);
const copies = Number(copiesArg ?? 1);
if (!repoRoot || !outDir || !Number.isInteger(copies) || copies < 1) {
  console.error("usage: node make-corpus.mjs <repo-root> <out-dir> <copies>");
  process.exit(1);
}

const found = execFileSync(
  "find",
  [path.join(repoRoot, "node_modules/.pnpm"), "-name", "*.js", "-size", "+20k", "-size", "-300k"],
  { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
)
  .split("\n")
  .filter((filePath) => filePath && !filePath.endsWith(".min.js"))
  .sort();

const candidates = [];
for (const file of found) {
  const text = fs.readFileSync(file, "utf8");
  if (text.length / text.split("\n").length > 500) continue;
  candidates.push({ file, text });
}

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(outDir, { recursive: true });

let bytes = 0;
let count = 0;
for (let copy = 0; copy < copies; copy++) {
  for (const [index, { file, text }] of candidates.entries()) {
    // Flat names keep directory walking out of the measurement.
    const copyTag = String(copy).padStart(2, "0");
    const indexTag = String(index).padStart(4, "0");
    fs.writeFileSync(path.join(outDir, `f${copyTag}_${indexTag}_${path.basename(file)}`), text);
    bytes += text.length;
    count++;
  }
}

console.log(
  `${count} files, ${(bytes / 1048576).toFixed(1)} MiB ` +
    `(${candidates.length} distinct sources x ${copies})`,
);
