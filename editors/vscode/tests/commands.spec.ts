import { deepStrictEqual, notEqual, strictEqual } from 'assert';
import {
  commands,
  Uri,
  window,
  workspace,
  WorkspaceEdit
} from 'vscode';
import {
  activateExtension,
  sleep,
  testSingleFolderMode,
  WORKSPACE_DIR
} from './test-helpers';

const fileUri = Uri.joinPath(WORKSPACE_DIR, 'debugger.js');

suiteSetup(async () => {
  await activateExtension();
});

teardown(async () => {
  const edit = new WorkspaceEdit();
  edit.deleteFile(fileUri, {
    ignoreIfNotExists: true,
  });
  await workspace.applyEdit(edit);
});

suite('commands', () => {
  testSingleFolderMode('listed commands', async () => {
    const oxcCommands = (await commands.getCommands(true)).filter(x => x.startsWith('oxc.'));

    const expectedCommands = [
      'oxc.showOutputChannel',
      'oxc.showOutputChannelFormatter',
    ];

    if (process.env.SKIP_LINTER_TEST !== 'true') {
      expectedCommands.push(
        'oxc.restartServer',
        'oxc.toggleEnable',
        'oxc.applyAllFixesFile',
        'oxc.fixAll',
      );
    }

    if (process.env.SKIP_FORMATTER_TEST !== 'true' && !process.env.SERVER_PATH_DEV?.includes('oxc_language_server')) {
      expectedCommands.push(
        'oxc.restartServerFormatter',
      );
    }

    deepStrictEqual(expectedCommands, oxcCommands);
  });

  testSingleFolderMode('oxc.showOutputChannel', async () => {
    await commands.executeCommand('oxc.showOutputChannel');
    await sleep(250);

    notEqual(window.activeTextEditor, undefined);
    const { uri } = window.activeTextEditor!.document;
    strictEqual(uri.toString(), 'output:oxc.oxc-vscode.Oxc%20%28Lint%29');

    await commands.executeCommand('workbench.action.closeActiveEditor');
  });

  testSingleFolderMode('oxc.showOutputChannelFormatter', async () => {
    await commands.executeCommand('oxc.showOutputChannelFormatter');
    await sleep(250);

    notEqual(window.activeTextEditor, undefined);
    const { uri } = window.activeTextEditor!.document;
    strictEqual(uri.toString(), 'output:oxc.oxc-vscode.Oxc%20%28Fmt%29');

    await commands.executeCommand('workbench.action.closeActiveEditor');
  });

  testSingleFolderMode('oxc.toggleEnable', async () => {
    if (process.env.SKIP_LINTER_TEST === 'true') {
      return;
    }
    const isEnabledBefore = workspace.getConfiguration('oxc').get<boolean>('enable');
    strictEqual(isEnabledBefore, true);

    await commands.executeCommand('oxc.toggleEnable');
    await sleep(500);

    const isEnabledAfter = workspace.getConfiguration('oxc').get<boolean>('enable');
    strictEqual(isEnabledAfter, false);

    // enable it for other tests
    await commands.executeCommand('oxc.toggleEnable');
    await sleep(500);
  });

  test('oxc.fixAll', async () => {
    // Skip tests if linter tests are disabled
    if (process.env.SKIP_LINTER_TEST === 'true') {
      return;
    }
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('/* 😊 */debugger;'),
      overwrite: true,
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await commands.executeCommand('oxc.fixAll', {
      uri: fileUri.toString(),
    });
    await workspace.saveAll();

    const content = await workspace.fs.readFile(fileUri);

    strictEqual(content.toString(), '/* 😊 */');
  });
});
