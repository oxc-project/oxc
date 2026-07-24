// oxlint-disable no-console

import { exec } from "node:child_process";
import { rmSync } from "node:fs";
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

await Promise.all(
  sources.map(async ({ name, repo, version }) => {
    const dest = join(externalsDir, name);
    rmSync(dest, { recursive: true, force: true });

    console.log(`Downloading ${name}@${version} fixtures...`);
    await execAsync(`pnpm exec degit ${repo}#${version} "${dest}"`, { cwd });
    console.log(`Done: ${name}@${version}`);
  }),
);
