import { deepStrictEqual, strictEqual } from 'assert';
import { Uri, workspace, WorkspaceEdit } from 'vscode';
import { WorkspaceConfig } from './WorkspaceConfig.js';

suite('Config', () => {
  setup(async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    const keys = ['lint.run', 'configPath', 'flags'];

    await Promise.all(keys.map(key => wsConfig.update(key, undefined)));
  });

  suiteTeardown(async () => {
    const WORKSPACE_DIR = workspace.workspaceFolders![0].uri.toString();
    const file = Uri.parse(WORKSPACE_DIR + '/.vscode/settings.json');
    const edit = new WorkspaceEdit();
    edit.deleteFile(file);
    await workspace.applyEdit(edit);
  });

  test('default values on initialization', () => {
    const config = new WorkspaceConfig(workspace.getConfiguration('oxc'));
    strictEqual(config.runTrigger, 'onType');
    strictEqual(config.configPath, null);
    deepStrictEqual(config.flags, {});
  });

  test('configPath defaults to null when using nested configs and configPath is empty', async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    await wsConfig.update('configPath', '');
    await wsConfig.update('flags', {});

    const config = new WorkspaceConfig(workspace.getConfiguration('oxc'));

    deepStrictEqual(config.flags, {});
    strictEqual(config.configPath, null);
  });

  test('configPath defaults to .oxlintrc.json when not using nested configs and configPath is empty', async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    await wsConfig.update('configPath', undefined);
    await wsConfig.update('flags', { disable_nested_config: '' });

    const config = new WorkspaceConfig(workspace.getConfiguration('oxc'));

    deepStrictEqual(config.flags, { disable_nested_config: '' });
    strictEqual(config.configPath, '.oxlintrc.json');
  });

  test('updating values updates the workspace configuration', async () => {
    const config = new WorkspaceConfig(workspace.getConfiguration('oxc'));

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateConfigPath('./somewhere'),
      config.updateFlags({ test: 'value' }),
    ]);

    const wsConfig = workspace.getConfiguration('oxc');

    strictEqual(wsConfig.get('lint.run'), 'onSave');
    strictEqual(wsConfig.get('configPath'), './somewhere');
    deepStrictEqual(wsConfig.get('flags'), { test: 'value' });
  });
});
