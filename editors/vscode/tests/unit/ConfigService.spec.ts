import { strictEqual } from 'assert';
import { Uri, workspace, WorkspaceEdit } from 'vscode';
import { ConfigService } from '../../client/ConfigService.js';
import { WORKSPACE_FOLDER } from '../test-helpers.js';

const conf = workspace.getConfiguration('oxc');

suite('ConfigService', () => {
  setup(async () => {
    const keys = ['path.server', 'path.oxlint', 'path.oxfmt', 'path.tsgolint'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  teardown(async () => {
    const keys = ['path.server', 'path.oxlint', 'path.oxfmt', 'path.tsgolint'];

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
    test('resolves relative server path with workspace folder', async () => {
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

    test('returns undefined for unsafe server path', async () => {
      const service = new ConfigService();
      await conf.update('path.oxfmt', '../unsafe/oxfmt');
      const unsafeServerPath = await service.getOxfmtServerBinPath();

      strictEqual(unsafeServerPath, undefined);
    });

    test('returns backslashes path on Windows', async () => {
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
    test('resolves relative server path with workspace folder', async () => {
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

    test('returns undefined for unsafe server path', async () => {
      const service = new ConfigService();
      await conf.update('path.oxlint', '../unsafe/oxlint');
      const unsafeServerPath = await service.getOxlintServerBinPath();

      strictEqual(unsafeServerPath, undefined);
    });

    test('returns backslashes path on Windows', async () => {
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

  suite('Binary Auto-Discovery', () => {
    const createTestBinary = async (binaryName: string, workspaceUri: Uri = WORKSPACE_FOLDER.uri) => {
      const binDir = Uri.joinPath(workspaceUri, 'node_modules', '.bin');
      const binaryUri = Uri.joinPath(binDir, binaryName);
      const edit = new WorkspaceEdit();
      edit.createFile(binaryUri, {
        contents: Buffer.from('#!/usr/bin/env node\nconsole.log("test");'),
        overwrite: true,
      });
      await workspace.applyEdit(edit);
      return binaryUri.fsPath;
    };

    const cleanupTestBinary = async (binaryName: string, workspaceUri: Uri = WORKSPACE_FOLDER.uri) => {
      const binDir = Uri.joinPath(workspaceUri, 'node_modules', '.bin');
      const binaryUri = Uri.joinPath(binDir, binaryName);
      const edit = new WorkspaceEdit();
      edit.deleteFile(binaryUri, { ignoreIfNotExists: true });
      await workspace.applyEdit(edit);
    };

    teardown(async () => {
      await cleanupTestBinary('oxlint');
      await cleanupTestBinary('oxfmt');
    });

    test('finds binary in workspace node_modules/.bin/', async () => {
      const expectedPath = await createTestBinary('oxlint');
      const service = new ConfigService();

      const binaryPath = await service.getOxlintServerBinPath();

      strictEqual(binaryPath, expectedPath);
    });

    test('returns undefined when binary not found', async () => {
      const service = new ConfigService();

      const binaryPath = await service.getOxlintServerBinPath();

      strictEqual(binaryPath, undefined);
    });

    test('finds oxfmt binary in workspace', async () => {
      const expectedPath = await createTestBinary('oxfmt');
      const service = new ConfigService();

      const binaryPath = await service.getOxfmtServerBinPath();

      strictEqual(binaryPath, expectedPath);
    });

    test('returns properly formatted file system path', async () => {
      await createTestBinary('oxlint');
      const service = new ConfigService();

      const binaryPath = await service.getOxlintServerBinPath();

      strictEqual(typeof binaryPath, 'string');
      if (process.platform === 'win32') {
        // On Windows, fsPath should use backslashes and have drive letter
        strictEqual(binaryPath!.includes('/'), false, 'Windows path should not contain forward slashes');
        strictEqual(binaryPath![1], ':', 'Windows path should have drive letter');
      } else {
        // On Unix, fsPath should use forward slashes
        strictEqual(binaryPath!.startsWith('/'), true, 'Unix path should start with /');
      }
    });
  });
});
