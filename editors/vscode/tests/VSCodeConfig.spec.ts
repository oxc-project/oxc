import { strictEqual } from 'assert';
import { workspace } from 'vscode';
import { VSCodeConfig } from '../client/VSCodeConfig.js';
import { testSingleFolderMode } from './test-helpers.js';

const conf = workspace.getConfiguration('oxc');

suite('VSCodeConfig', () => {
  setup(async () => {
    const keys = ['enable', 'requireConfig', 'trace.server', 'path.server'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  teardown(async () => {
    const keys = ['enable', 'requireConfig', 'trace.server', 'path.server'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  testSingleFolderMode('default values on initialization', () => {
    const config = new VSCodeConfig();

    strictEqual(config.enable, true);
    strictEqual(config.requireConfig, false);
    strictEqual(config.trace, 'off');
    strictEqual(config.binPath, '');
  });

  testSingleFolderMode('updating values updates the workspace configuration', async () => {
    const config = new VSCodeConfig();

    await Promise.all([
      config.updateEnable(false),
      config.updateRequireConfig(true),
      config.updateTrace('messages'),
      config.updateBinPath('./binary'),
    ]);

    const wsConfig = workspace.getConfiguration('oxc');

    strictEqual(wsConfig.get('enable'), false);
    strictEqual(wsConfig.get('requireConfig'), true);
    strictEqual(wsConfig.get('trace.server'), 'messages');
    strictEqual(wsConfig.get('path.server'), './binary');
  });
});
