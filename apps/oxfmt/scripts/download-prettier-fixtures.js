// oxlint-disable no-console

import { execSync } from "node:child_process";
import { rmSync } from "node:fs";
import { join } from "node:path";
import pkg from "../package.json" with { type: "json" };

const version = pkg.dependencies.prettier;

const root = join(import.meta.dirname, "..");
const dest = join(root, "prettier-fixtures");

rmSync(dest, { recursive: true, force: true });

console.log(`Downloading prettier@${version} tests/format fixtures...`);
execSync(`pnpm exec degit prettier/prettier/tests/format#${version} "${dest}"`, {
  stdio: "inherit",
  cwd: root,
});
console.log("Done!");
