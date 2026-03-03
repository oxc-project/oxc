// oxlint-disable no-console

import { execSync } from "node:child_process";
import { copyFileSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

const oxlintDirPath = join(import.meta.dirname, ".."),
  srcDirPath = join(oxlintDirPath, "src-js"),
  distDirPath = join(oxlintDirPath, "dist"),
  distPkgPluginsDirPath = join(oxlintDirPath, "dist-pkg-plugins"),
  distPkgPluginEslintDirPath = join(oxlintDirPath, "dist-pkg-plugin-eslint");

// Delete `dist-pkg-plugins` directory
console.log("Deleting `dist-pkg-plugins` directory...");
rmSync(distPkgPluginsDirPath, { recursive: true, force: true });

// Delete `dist-pkg-plugin-eslint` directory
console.log("Deleting `dist-pkg-plugin-eslint` directory...");
rmSync(distPkgPluginEslintDirPath, { recursive: true, force: true });

// Generate plugin-eslint files
console.log("Generating plugin-eslint files...");
execSync("node scripts/generate-plugin-eslint.ts", {
  stdio: "inherit",
  cwd: oxlintDirPath,
});

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
