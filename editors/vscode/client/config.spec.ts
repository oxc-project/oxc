import { deepStrictEqual, strictEqual } from 'assert';
import { Uri, workspace, WorkspaceEdit } from 'vscode';
import { Config } from './Config.js';

suite('Config', () => {
  setup(async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    const keys = ['lint.run', 'enable', 'trace.server', 'configPath', 'path.server', 'flags'];

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
    const config = new Config();

    strictEqual(config.runTrigger, 'onType');
    strictEqual(config.enable, true);
    strictEqual(config.trace, 'off');
    strictEqual(config.configPath, '');
    strictEqual(config.binPath, '');
    deepStrictEqual(config.flags, {});
  });

  test('configPath defaults to empty string when using nested configs and configPath is empty', async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    await wsConfig.update('configPath', '');
    await wsConfig.update('flags', {});

    const config = new Config();

    deepStrictEqual(config.flags, {});
    strictEqual(config.configPath, '');
  });

  test('configPath defaults to .oxlintrc.json when not using nested configs and configPath is empty', async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    await wsConfig.update('configPath', '');
    await wsConfig.update('flags', { disable_nested_config: '' });

    const config = new Config();

    deepStrictEqual(config.flags, { disable_nested_config: '' });
    strictEqual(config.configPath, '.oxlintrc.json');
  });

  test('updating values updates the workspace configuration', async () => {
    const config = new Config();

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateEnable(false),
      config.updateTrace('messages'),
      config.updateConfigPath('./somewhere'),
      config.updateBinPath('./binary'),
      config.updateFlags({ test: 'value' }),
    ]);

    const wsConfig = workspace.getConfiguration('oxc');

    strictEqual(wsConfig.get('lint.run'), 'onSave');
    strictEqual(wsConfig.get('enable'), false);
    strictEqual(wsConfig.get('trace.server'), 'messages');
    strictEqual(wsConfig.get('configPath'), './somewhere');
    strictEqual(wsConfig.get('path.server'), './binary');
    deepStrictEqual(wsConfig.get('flags'), { test: 'value' });
  });
});
