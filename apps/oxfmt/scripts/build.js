// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync } from "node:fs";
import { join } from "node:path";

const oxfmtDirPath = join(import.meta.dirname, ".."),
  distDirPath = join(oxfmtDirPath, "dist");

// Build with tsdown
console.log("Building with tsdown...");
execSync("pnpm tsdown", { stdio: "inherit", cwd: oxfmtDirPath });

// Copy native `.node` files from `src-js`
console.log("Copying `.node` files...");

for (const filename of readdirSync(join(oxfmtDirPath, "src-js"))) {
  if (!filename.endsWith(".node")) continue;
  copyFile(join(oxfmtDirPath, "src-js", filename), join(distDirPath, filename));
}

console.log("Build complete!");

/**
 * Copy a file, creating parent directories if needed.
 * @param {string} srcPath - Source file path, absolute
 * @param {string} destPath - Destination file path, absolute
 * @returns {void}
 */
function copyFile(srcPath, destPath) {
  mkdirSync(join(destPath, ".."), { recursive: true });
  copyFileSync(srcPath, destPath);
  console.log(`- Copied ${srcPath.split("/").pop()}`);
}
