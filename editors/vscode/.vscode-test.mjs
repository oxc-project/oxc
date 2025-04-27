import { defineConfig } from '@vscode/test-cli';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';

const multiRootWorkspaceFile = './multi-root.test.code-workspace';

mkdirSync('./test_workspace', { recursive: true });
mkdirSync('./test_workspace_second', { recursive: true });

const multiRootWorkspaceConfig = {
  'folders': [
    { 'path': 'test_workspace' },
    { 'path': 'test_workspace_second' },
  ],
};
writeFileSync(multiRootWorkspaceFile, JSON.stringify(multiRootWorkspaceConfig, null, 2));

const ext = process.platform === 'win32' ? '.exe' : '';

export default defineConfig({
  tests: [
    {
      files: 'out/**/*.spec.js',
      workspaceFolder: './test_workspace',
      launchArgs: [
        // This disables all extensions except the one being testing
        '--disable-extensions',
      ],
      env: {
        SINGLE_FOLDER_WORKSPACE: 'true',
        SERVER_PATH_DEV: path.resolve(
          import.meta.dirname,
          `./target/debug/oxc_language_server${ext}`,
        ),
      },
      mocha: {
        timeout: 10_000,
      },
    },
    {
      files: 'out/**/*.spec.js',
      workspaceFolder: multiRootWorkspaceFile,
      launchArgs: [
        // This disables all extensions except the one being testing
        '--disable-extensions',
      ],
      env: {
        MULTI_FOLDER_WORKSPACE: 'true',
        SERVER_PATH_DEV: path.resolve(
          import.meta.dirname,
          `./target/debug/oxc_language_server${ext}`,
        ),
      },
      mocha: {
        timeout: 10_000,
      },
    },
  ],
});
