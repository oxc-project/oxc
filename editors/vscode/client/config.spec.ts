import { strictEqual } from 'assert';
import { workspace } from 'vscode';
import { Config } from './Config.js';

suite('Config', () => {
  setup(async () => {
    const wsConfig = workspace.getConfiguration('oxc');
    const keys = ['lint.run', 'enable', 'trace.server', 'configPath', 'path.server'];

    await Promise.all(keys.map(key => wsConfig.update(key, undefined)));
  });

  test('default values on initialization', () => {
    const config = new Config();

    strictEqual(config.runTrigger, 'onType');
    strictEqual(config.enable, true);
    strictEqual(config.trace, 'off');
    strictEqual(config.configPath, '.oxlintrc.json');
    strictEqual(config.binPath, '');
  });

  test('updating values updates the workspace configuration', async () => {
    const config = new Config();

    await Promise.all([
      config.updateRunTrigger('onSave'),
      config.updateEnable(false),
      config.updateTrace('messages'),
      config.updateConfigPath('./somewhere'),
      config.updateBinPath('./binary'),
    ]);

    const wsConfig = workspace.getConfiguration('oxc');

    strictEqual(wsConfig.get('lint.run'), 'onSave');
    strictEqual(wsConfig.get('enable'), false);
    strictEqual(wsConfig.get('trace.server'), 'messages');
    strictEqual(wsConfig.get('configPath'), './somewhere');
    strictEqual(wsConfig.get('path.server'), './binary');
  });
});
