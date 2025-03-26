import { defineConfig } from '@vscode/test-cli';
import { existsSync, mkdirSync } from 'node:fs';

if (!existsSync('./test_workspace')) {
  mkdirSync('./test_workspace');
}

export default defineConfig({
  files: 'out/**/*.spec.js',
  workspaceFolder: './test_workspace',
});
