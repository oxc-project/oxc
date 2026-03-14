// oxlint-disable no-console

import { execSync } from "node:child_process";
import { rmSync } from "node:fs";
import { join } from "node:path";
import pkg from "../package.json" with { type: "json" };

const fixturesDir = join(import.meta.dirname, "fixtures");
const cwd = join(import.meta.dirname, "..");

const sources = [
  {
    name: "prettier",
    repo: "prettier/prettier/tests/format",
    version: pkg.dependencies.prettier,
  },
  {
    name: "vue-vben-admin",
    repo: "vbenjs/vue-vben-admin/packages",
    version: "main",
  },
  // {
  //   name: "plugin-svelte",
  //   repo: "sveltejs/prettier-plugin-svelte/tests",
  //   version: pkg.dependencies["prettier-plugin-svelte"],
  // },
];

for (const { name, repo, version } of sources) {
  const dest = join(fixturesDir, name);
  rmSync(dest, { recursive: true, force: true });

  console.log(`Downloading ${name}@${version} fixtures...`);
  execSync(`pnpm exec degit ${repo}#${version} "${dest}"`, { stdio: "inherit", cwd });
  console.log(`Done: ${name}\n`);
}
