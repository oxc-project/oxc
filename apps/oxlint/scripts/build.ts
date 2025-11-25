// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const oxlintDirPath = join(import.meta.dirname, ".."),
  srcDirPath = join(oxlintDirPath, "src-js"),
  distDirPath = join(oxlintDirPath, "dist");

// Modify `bindings.js` to use correct package names
console.log("Modifying bindings.js...");
const bindingsPath = join(oxlintDirPath, "src-js/bindings.js");
let bindingsJs = readFileSync(bindingsPath, "utf8");
bindingsJs = bindingsJs.replace(/require\('@oxlint\/binding-(.+?)'\)/g, (_, name) => {
  name = name.replace(/-msvc(\/|$)/g, "$1");
  return `require('@oxlint/${name}')`;
});
writeFileSync(bindingsPath, bindingsJs);

// Build with tsdown
console.log("Building with tsdown...");
execSync("pnpm tsdown", { stdio: "inherit", cwd: oxlintDirPath });

// Delete `cli.d.ts`
console.log("Deleting cli.d.ts...");
rmSync(join(distDirPath, "cli.d.ts"));

// Copy native `.node` files from `src-js`
console.log("Copying `.node` files...");
for (const filename of readdirSync(srcDirPath)) {
  if (!filename.endsWith(".node")) continue;
  const srcPath = join(srcDirPath, filename);
  copyFileSync(srcPath, join(distDirPath, filename));
}

console.log("Build complete!");
