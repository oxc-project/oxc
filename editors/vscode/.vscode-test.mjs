import { defineConfig } from '@vscode/test-cli';

export default defineConfig({
  files: 'out/**/*.spec.js',
  workspaceFolder: './test_workspace',
});
