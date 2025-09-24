import { strictEqual } from 'assert';
import { workspace } from 'vscode';
import { ConfigService } from '../client/ConfigService.js';
import { testSingleFolderMode, WORKSPACE_FOLDER } from './test-helpers.js';

const conf = workspace.getConfiguration('oxc');

suite('ConfigService', () => {
  setup(async () => {
    const keys = ['path.server'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  teardown(async () => {
    const keys = ['path.server'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  testSingleFolderMode('resolves relative server path with workspace folder', async () => {
    const service = new ConfigService();
    const nonDefinedServerPath = service.getUserServerBinPath();

    strictEqual(nonDefinedServerPath, undefined);

    await conf.update('path.server', '/absolute/oxc_language_server');
    const absoluteServerPath = service.getUserServerBinPath();

    strictEqual(absoluteServerPath, '/absolute/oxc_language_server');

    await conf.update('path.server', './relative/oxc_language_server');
    const relativeServerPath = service.getUserServerBinPath();

    strictEqual(relativeServerPath, WORKSPACE_FOLDER.uri.path + '/relative/oxc_language_server');
  });
});
