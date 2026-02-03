// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

const oxlintDirPath = join(import.meta.dirname, ".."),
  srcDirPath = join(oxlintDirPath, "src-js"),
  distDirPath = join(oxlintDirPath, "dist"),
  distPkgPluginsDirPath = join(oxlintDirPath, "dist-pkg-plugins"),
  pkgPluginsDirPath = join(oxlintDirPath, "../../npm/oxlint-plugins");

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

// Copy files to `@oxlint/plugins` package
console.log("Moving files to `@oxlint/plugins` package...");

const files = [
  { src: "esm/index.js", dest: "index.js" },
  { src: "esm/index.d.ts", dest: "index.d.ts" },
  { src: "cjs/index.cjs", dest: "index.cjs" },
];

for (const { src, dest } of files) {
  copyFileSync(join(distPkgPluginsDirPath, src), join(pkgPluginsDirPath, dest));
}

rmSync(distPkgPluginsDirPath, { recursive: true });

console.log("Build complete!");
