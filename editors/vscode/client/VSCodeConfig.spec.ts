import { strictEqual } from 'assert';
import { Uri, workspace, WorkspaceEdit } from 'vscode';
import { VSCodeConfig } from './VSCodeConfig.js';

const conf = workspace.getConfiguration('oxc');

suite('Config', () => {
  setup(async () => {
    const keys = ['enable', 'trace.server', 'path.server'];

    await Promise.all(keys.map(key => conf.update(key, undefined)));
  });

  suiteTeardown(async () => {
    const WORKSPACE_DIR = workspace.workspaceFolders![0].uri.toString();
    const file = Uri.parse(WORKSPACE_DIR + '/.vscode/settings.json');
    const edit = new WorkspaceEdit();
    edit.deleteFile(file);
    await workspace.applyEdit(edit);
  });

  test('default values on initialization', () => {
    const config = new VSCodeConfig(conf);

    strictEqual(config.enable, true);
    strictEqual(config.trace, 'off');
    strictEqual(config.binPath, '');
  });

  test('updating values updates the workspace configuration', async () => {
    const config = new VSCodeConfig(conf);

    await Promise.all([
      config.updateEnable(false),
      config.updateTrace('messages'),
      config.updateBinPath('./binary'),
    ]);

    const wsConfig = workspace.getConfiguration('oxc');

    strictEqual(wsConfig.get('enable'), false);
    strictEqual(wsConfig.get('trace.server'), 'messages');
    strictEqual(wsConfig.get('path.server'), './binary');
  });
});
