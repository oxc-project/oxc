import { deepStrictEqual, strictEqual } from 'assert';
import { ConfigurationTarget, workspace } from 'vscode';
import { WorkspaceConfig } from '../client/WorkspaceConfig.js';
import { WORKSPACE_FOLDER } from './test-helpers.js';

suite('WorkspaceConfig', () => {
  setup(async () => {
    const workspaceConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);
    const globalConfig = workspace.getConfiguration('oxc');
    const keys = ['lint.run', 'configPath', 'flags'];

    await Promise.all(keys.map(key => workspaceConfig.update(key, undefined, ConfigurationTarget.WorkspaceFolder)));
    // VSCode will not save different workspace configuration inside a `.code-workspace` file.
    // Do not fail, we will make sure the global config is empty too.
    await Promise.all(keys.map(key => globalConfig.update(key, undefined)));
  });

  teardown(async () => {
    const workspaceConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);
    const globalConfig = workspace.getConfiguration('oxc');
    const keys = ['lint.run', 'configPath', 'flags'];

    await Promise.all(keys.map(key => workspaceConfig.update(key, undefined, ConfigurationTarget.WorkspaceFolder)));
    // VSCode will not save different workspace configuration inside a `.code-workspace` file.
    // Do not fail, we will make sure the global config is empty too.
    await Promise.all(keys.map(key => globalConfig.update(key, undefined)));
  });

  test('default values on initialization', () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);
    strictEqual(config.runTrigger, 'onType');
    strictEqual(config.configPath, null);
    deepStrictEqual(config.flags, {});
  });

  test('configPath defaults to null when using nested configs and configPath is empty', async () => {
    const wsConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);
    await wsConfig.update('configPath', '', ConfigurationTarget.WorkspaceFolder);
    await wsConfig.update('flags', {}, ConfigurationTarget.WorkspaceFolder);

    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    deepStrictEqual(config.flags, {});
    strictEqual(config.configPath, null);
  });

  test('configPath defaults to .oxlintrc.json when not using nested configs and configPath is empty', async () => {
    const wsConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);
    await wsConfig.update('configPath', undefined, ConfigurationTarget.WorkspaceFolder);
    await wsConfig.update('flags', { disable_nested_config: '' }, ConfigurationTarget.WorkspaceFolder);

    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    deepStrictEqual(config.flags, { disable_nested_config: '' });
    strictEqual(config.configPath, '.oxlintrc.json');
  });

  test('updating values updates the workspace configuration', async () => {
    const config = new WorkspaceConfig(WORKSPACE_FOLDER);

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateConfigPath('./somewhere'),
      config.updateFlags({ test: 'value' }),
    ]);

    const wsConfig = workspace.getConfiguration('oxc', WORKSPACE_FOLDER);

    strictEqual(wsConfig.get('lint.run'), 'onSave');
    strictEqual(wsConfig.get('configPath'), './somewhere');
    deepStrictEqual(wsConfig.get('flags'), { test: 'value' });
  });
});
