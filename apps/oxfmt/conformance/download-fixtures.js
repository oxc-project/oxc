// oxlint-disable no-console

import { exec } from "node:child_process";
import { rmSync } from "node:fs";
import { join } from "node:path";
import { promisify } from "node:util";
import pkg from "../package.json" with { type: "json" };

const execAsync = promisify(exec);

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
    version: "v5.6.0",
  },
  {
    name: "webawesome",
    repo: "shoelace-style/webawesome/packages/webawesome/src/components",
    version: "v3.5.0",
  },
  // {
  //   name: "plugin-svelte",
  //   repo: "sveltejs/prettier-plugin-svelte/tests",
  //   version: pkg.dependencies["prettier-plugin-svelte"],
  // },
];

await Promise.all(
  sources.map(async ({ name, repo, version }) => {
    const dest = join(fixturesDir, name);
    rmSync(dest, { recursive: true, force: true });

    console.log(`Downloading ${name}@${version} fixtures...`);
    await execAsync(`pnpm exec degit ${repo}#${version} "${dest}"`, { cwd });
    console.log(`Done: ${name}@${version}`);
  }),
);
