// oxlint-disable no-console, no-await-in-loop

import { exec } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { promisify } from "node:util";
import pkg from "../package.json" with { type: "json" };

const execAsync = promisify(exec);

const externalsDir = join(import.meta.dirname, "fixtures", "externals");
const cwd = join(import.meta.dirname, "..");

const sources = [
  // xxx-in-js
  {
    name: "prettier",
    repo: "prettier/prettier/tests/format",
    version: pkg.dependencies.prettier,
  },
  // js-in-vue
  {
    name: "vue-vben-admin",
    repo: "vbenjs/vue-vben-admin/packages",
    version: "v5.6.0",
  },
  // html-in-js
  {
    name: "webawesome",
    repo: "shoelace-style/webawesome/packages/webawesome/src/components",
    version: "v3.6.0",
  },
  // svelte
  {
    name: "plugin-svelte",
    repo: "sveltejs/prettier-plugin-svelte/test/formatting/samples",
    version: `prettier-plugin-svelte@${pkg.dependencies["prettier-plugin-svelte"]}`,
  },
  // graphql
  {
    name: "gitlab",
    repo: "gitlabhq/gitlabhq/app/assets",
    version: "v16.9.0",
  },
  // less
  {
    name: "ng-zorro-antd",
    repo: "NG-ZORRO/ng-zorro-antd",
    version: "21.3.1",
  },
  // yaml
  {
    name: "aws-cloudformation-templates",
    repo: "aws-cloudformation/aws-cloudformation-templates",
    // No maintained tags; pin to a commit (2026-07 main)
    version: "a0f43bc6d20813052892546f445037cf84c75b54",
  },
  {
    name: "gitlab-ci-templates",
    repo: "gitlabhq/gitlabhq/lib/gitlab/ci/templates",
    version: "v16.9.0",
  },
  // css (css modules)
  {
    name: "mantine",
    repo: "mantinedev/mantine/packages/@mantine",
    version: "9.3.2",
  },
  {
    name: "docusaurus",
    repo: "facebook/docusaurus/packages/docusaurus-theme-classic/src",
    version: "v3.9.2",
  },
];

// Group sources by repository and download each group sequentially.
// Parallel `degit` calls for the same repo+ref share a single tarball cache path;
// one process sees the other's partially written tarball, fails to extract it,
// and silently falls back to `git clone` which ignores the subdirectory,
// dumping the entire repository into the fixture directory.
const sourcesByRepo = new Map();
for (const source of sources) {
  const repoKey = source.repo.split("/").slice(0, 2).join("/");
  if (!sourcesByRepo.has(repoKey)) sourcesByRepo.set(repoKey, []);
  sourcesByRepo.get(repoKey).push(source);
}

await Promise.all(
  [...sourcesByRepo.values()].map(async (group) => {
    for (const { name, repo, version } of group) {
      const dest = join(externalsDir, name);

      // Stamp-based skip (same scheme as `oxc_formatter_tests`' suite provisioning):
      // the stamp is written last, so a half-downloaded tree is always re-done.
      const stamp = join(dest, ".version");
      const pin = `${repo}#${version}`;
      if (existsSync(stamp) && readFileSync(stamp, "utf8").trim() === pin) {
        console.log(`Up-to-date: ${name}@${version}`);
        continue;
      }
      rmSync(dest, { recursive: true, force: true });

      console.log(`Downloading ${name}@${version} fixtures...`);
      await execAsync(`pnpm exec degit ${repo}#${version} "${dest}"`, { cwd });
      writeFileSync(stamp, pin);
      console.log(`Done: ${name}@${version}`);
    }
  }),
);
