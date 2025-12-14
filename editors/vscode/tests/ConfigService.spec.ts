import { strictEqual } from 'assert';
import { workspace } from 'vscode';
import { ConfigService } from '../client/ConfigService.js';
import { testSingleFolderMode, WORKSPACE_FOLDER } from './test-helpers.js';

const conf = workspace.getConfiguration('oxc');

suite('ConfigService', () => {
  setup(async () => {
    const keys = ['path.server', 'path.oxlint', 'path.oxfmt'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  teardown(async () => {
    const keys = ['path.server', 'path.oxlint', 'path.oxfmt'];

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

  suite('getOxfmtServerBinPath', () => {
    testSingleFolderMode('resolves relative server path with workspace folder', async () => {
      const service = new ConfigService();
      const nonDefinedServerPath = await service.getOxfmtServerBinPath();

      strictEqual(nonDefinedServerPath, undefined);

      await conf.update('path.oxfmt', '/absolute/oxfmt');
      const absoluteServerPath = await service.getOxfmtServerBinPath();

      strictEqual(absoluteServerPath, '/absolute/oxfmt');

      await conf.update('path.oxfmt', './relative/oxfmt');
      const relativeServerPath = await service.getOxfmtServerBinPath();

      const workspace_path = getWorkspaceFolderPlatformSafe();
      strictEqual(relativeServerPath, `${workspace_path}/relative/oxfmt`);
    });

    testSingleFolderMode('returns undefined for unsafe server path', async () => {
      const service = new ConfigService();
      await conf.update('path.oxfmt', '../unsafe/oxfmt');
      const unsafeServerPath = await service.getOxfmtServerBinPath();

      strictEqual(unsafeServerPath, undefined);
    });

    testSingleFolderMode('returns backslashes path on Windows', async () => {
      if (process.platform !== 'win32') {
        return;
      }
      const service = new ConfigService();
      await conf.update('path.oxfmt', './relative/oxfmt');
      const relativeServerPath = await service.getOxfmtServerBinPath();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      strictEqual(workspace_path[1], ':', 'The test workspace folder must be an absolute path with a drive letter on Windows');
      strictEqual(relativeServerPath, `${workspace_path}\\relative\\oxfmt`);
    });
  });

  suite('getOxlintServerBinPath', () => {
    testSingleFolderMode('resolves relative server path with workspace folder', async () => {
      const service = new ConfigService();
      const nonDefinedServerPath = await service.getOxlintServerBinPath();

      strictEqual(nonDefinedServerPath, undefined);

      await conf.update('path.oxlint', '/absolute/oxlint');
      const absoluteServerPath = await service.getOxlintServerBinPath();

      strictEqual(absoluteServerPath, '/absolute/oxlint');

      await conf.update('path.oxlint', './relative/oxlint');
      const relativeServerPath = await service.getOxlintServerBinPath();

      const workspace_path = getWorkspaceFolderPlatformSafe();
      strictEqual(relativeServerPath, `${workspace_path}/relative/oxlint`);
    });

    testSingleFolderMode('returns undefined for unsafe server path', async () => {
      const service = new ConfigService();
      await conf.update('path.oxlint', '../unsafe/oxlint');
      const unsafeServerPath = await service.getOxlintServerBinPath();

      strictEqual(unsafeServerPath, undefined);
    });

    testSingleFolderMode('returns backslashes path on Windows', async () => {
      if (process.platform !== 'win32') {
        return;
      }
      const service = new ConfigService();
      await conf.update('path.oxlint', './relative/oxlint');
      const relativeServerPath = await service.getOxlintServerBinPath();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      strictEqual(workspace_path[1], ':', 'The test workspace folder must be an absolute path with a drive letter on Windows');
      strictEqual(relativeServerPath, `${workspace_path}\\relative\\oxlint`);
    });
  });
});
