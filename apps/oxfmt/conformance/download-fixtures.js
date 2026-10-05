// oxlint-disable no-console, no-await-in-loop

import { exec, spawn } from "node:child_process";
import { once } from "node:events";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createInterface } from "node:readline";
import { promisify } from "node:util";
import pkg from "../package.json" with { type: "json" };

const execAsync = promisify(exec);

const externalsDir = join(import.meta.dirname, "fixtures", "externals");

// `repo` is `<owner>/<name>` plus the directory to take, `version` is any ref (a tag or a commit)
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
  // jsdoc
  {
    name: "svelte",
    repo: "sveltejs/svelte/packages/svelte/src",
    version: "svelte@5.57.0",
  },
];

// Group sources by archive, so an archive shared by several sources downloads once.
const sourcesByArchive = Map.groupBy(
  sources,
  ({ repo, version }) => `${repo.split("/").slice(0, 2).join("/")}#${version}`,
);

await Promise.all(
  [...sourcesByArchive.values()].map(async (group) => {
    // Stamp-based skip (same scheme as `oxc_formatter_tests`' suite provisioning):
    // the stamp is written last, so a half-downloaded tree is always re-done.
    const stale = group.filter(({ name, repo, version }) => {
      const stamp = join(externalsDir, name, ".version");
      const upToDate =
        existsSync(stamp) && readFileSync(stamp, "utf8").trim() === `${repo}#${version}`;
      if (upToDate) console.log(`Up-to-date: ${name}@${version}`);
      return !upToDate;
    });
    if (stale.length === 0) return;

    const [owner, repoName] = group[0].repo.split("/");
    const { version } = group[0];
    console.log(`Downloading ${owner}/${repoName}@${version}...`);
    const tmp = mkdtempSync(join(tmpdir(), "oxfmt-fixtures-"));
    const tarball = join(tmp, "archive.tar.gz");
    await execAsync(
      `curl -fsSL -o "${tarball}" https://codeload.github.com/${owner}/${repoName}/tar.gz/${version}`,
    );
    const top = await topDirectory(tarball);

    for (const { name, repo } of stale) {
      const dest = join(externalsDir, name);
      rmSync(dest, { recursive: true, force: true });
      mkdirSync(dest, { recursive: true });
      const subdir = repo.split("/").slice(2);
      await execAsync(
        `tar -xzf "${tarball}" -C "${dest}" --strip-components=${subdir.length + 1} "${[top, ...subdir].join("/")}"`,
      );
      writeFileSync(join(dest, ".version"), `${repo}#${version}`);
      console.log(`Done: ${name}@${version}`);
    }
    rmSync(tmp, { recursive: true });
  }),
);

/** The archive's top directory, GitHub's `<name>-<ref>` with the ref normalized (e.g. no leading `v`). */
async function topDirectory(tarball) {
  const tar = spawn("tar", ["-tzf", tarball]);
  const [line] = await once(createInterface({ input: tar.stdout }), "line");
  tar.kill();
  return line.split("/")[0];
}
