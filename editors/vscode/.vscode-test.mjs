import { defineConfig } from "@vscode/test-cli";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const multiRootWorkspaceFile = "./multi-root.test.code-workspace";

mkdirSync("./test_workspace", { recursive: true });
mkdirSync("./test_workspace_second", { recursive: true });

const multiRootWorkspaceConfig = {
  folders: [{ path: "test_workspace" }, { path: "test_workspace_second" }],
};
writeFileSync(multiRootWorkspaceFile, JSON.stringify(multiRootWorkspaceConfig, null, 2));

const ext = process.platform === "win32" ? ".exe" : "";

const baseTest = {
  files: "out/**/*.spec.js",
  workspaceFolder: "./test_workspace",
  launchArgs: [
    // This disables all extensions except the one being tested
    "--disable-extensions",
  ],
  mocha: {
    timeout: 10_000,
  },
};

const allTestSuites = new Map([
  [
    "single-folder",
    {
      ...baseTest,
      env: {
        SINGLE_FOLDER_WORKSPACE: "true",
        SERVER_PATH_DEV: path.resolve(
          import.meta.dirname,
          `./target/debug/oxc_language_server${ext}`,
        ),
      },
    },
  ],
  [
    "multi-root",
    {
      ...baseTest,
      workspaceFolder: multiRootWorkspaceFile,
      env: {
        MULTI_FOLDER_WORKSPACE: "true",
        SERVER_PATH_DEV: path.resolve(
          import.meta.dirname,
          `./target/debug/oxc_language_server${ext}`,
        ),
      },
    },
  ],
  [
    "oxlint-lsp",
    {
      ...baseTest,
      env: {
        SINGLE_FOLDER_WORKSPACE: "true",
        SERVER_PATH_DEV: path.resolve(import.meta.dirname, `../../apps/oxlint/dist/cli.js`),
        SKIP_FORMATTER_TEST: "true",
      },
    },
  ],
  [
    "oxfmt-lsp",
    {
      ...baseTest,
      env: {
        SINGLE_FOLDER_WORKSPACE: "true",
        SERVER_PATH_DEV: path.resolve(import.meta.dirname, `../../apps/oxfmt/dist/cli.js`),
        SKIP_LINTER_TEST: "true",
      },
    },
  ],
]);

export default defineConfig({
  tests: process.env.TEST_SUITE
    ? [allTestSuites.get(process.env.TEST_SUITE)]
    : Array.from(allTestSuites.values()),
});
