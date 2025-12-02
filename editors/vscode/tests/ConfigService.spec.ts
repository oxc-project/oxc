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

  const getWorkspaceFolderPlatformSafe = () => {
    let workspace_path = WORKSPACE_FOLDER.uri.path;
    if (process.platform === 'win32') {
      workspace_path = workspace_path.replaceAll('/', '\\');
      if (workspace_path.startsWith('\\')) {
        workspace_path = workspace_path.slice(1);
      }
    }
    return workspace_path;
  };

  suite('getUserServerBinPath', () => {
    testSingleFolderMode('resolves relative server path with workspace folder', async () => {
      const service = new ConfigService();
      const nonDefinedServerPath = service.getUserServerBinPath();

      strictEqual(nonDefinedServerPath, undefined);

      await conf.update('path.server', '/absolute/oxc_language_server');
      const absoluteServerPath = service.getUserServerBinPath();

      strictEqual(absoluteServerPath, '/absolute/oxc_language_server');

      await conf.update('path.server', './relative/oxc_language_server');
      const relativeServerPath = service.getUserServerBinPath();

      const workspace_path = getWorkspaceFolderPlatformSafe();
      strictEqual(relativeServerPath, `${workspace_path}/relative/oxc_language_server`);
    });

    testSingleFolderMode('returns undefined for unsafe server path', async () => {
      const service = new ConfigService();
      await conf.update('path.server', '../unsafe/oxc_language_server');
      const unsafeServerPath = service.getUserServerBinPath();

      strictEqual(unsafeServerPath, undefined);
    });

   testSingleFolderMode('returns backslashes path on Windows', async () => {
      if (process.platform !== 'win32') {
        return;
      }
      const service = new ConfigService();
      await conf.update('path.server', './relative/oxc_language_server');
      const relativeServerPath = service.getUserServerBinPath();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      strictEqual(workspace_path[1], ':', 'The test workspace folder must be an absolute path with a drive letter on Windows');
      strictEqual(relativeServerPath, `${workspace_path}\\relative\\oxc_language_server`);
    });
  });
});
