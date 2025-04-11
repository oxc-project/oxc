import { defineConfig } from '@vscode/test-cli';
import { existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';

if (!existsSync('./test_workspace')) {
  mkdirSync('./test_workspace');
}

const ext = process.platform === 'win32' ? '.exe' : '';

export default defineConfig({
  tests: [{
    files: 'out/**/*.spec.js',
    workspaceFolder: './test_workspace',
    launchArgs: [
      // This disables all extensions except the one being testing
      '--disable-extensions',
    ],
    env: {
      SERVER_PATH_DEV: path.resolve(import.meta.dirname, `./target/debug/oxc_language_server${ext}`),
    },
    mocha: {
      timeout: 10_000,
    },
  }],
});
