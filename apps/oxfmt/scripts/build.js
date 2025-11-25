// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const oxfmtDirPath = join(import.meta.dirname, ".."),
  distDirPath = join(oxfmtDirPath, "dist");

// Modify `bindings.js` to use correct package names
console.log("Modifying bindings.js...");
const bindingsPath = join(oxfmtDirPath, "src-js/bindings.js");
let bindingsJs = readFileSync(bindingsPath, "utf8");
bindingsJs = bindingsJs.replace(/require\('@oxfmt\/binding-(.+?)'\)/g, (_, name) => {
  name = name.replace(/-msvc(\/|$)/g, "$1");
  return `require('@oxfmt/${name}')`;
});
writeFileSync(bindingsPath, bindingsJs);

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
