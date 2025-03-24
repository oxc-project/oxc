import { deepStrictEqual, notEqual, strictEqual } from 'assert';
import { commands, extensions, window } from 'vscode';

// const WORKSPACE_DIR = workspace.workspaceFolders![0].uri.toString();
// const filePath = WORKSPACE_DIR + '/debugger.js';
// const fileUri = Uri.parse(filePath);

const sleep = (time: number) => new Promise((r) => setTimeout(r, time));

suite('commands', () => {
  setup(async () => {
    const ext = extensions.getExtension('oxc.oxc-vscode')!;
    await ext.activate();
  });

  // suiteTeardown(async () => {
  //   const edit = new WorkspaceEdit();
  //   edit.deleteFile(fileUri);
  //   await workspace.applyEdit(edit);
  // });

  test('listed commands', async () => {
    const oxcCommands = (await commands.getCommands(true)).filter(x => x.startsWith('oxc.'));

    deepStrictEqual([
      'oxc.restartServer',
      'oxc.showOutputChannel',
      'oxc.toggleEnable',
      'oxc.applyAllFixesFile',
      'oxc.fixAll',
    ], oxcCommands);
  });

  test('oxc.showOutputChannel', async () => {
    await commands.executeCommand('oxc.showOutputChannel');
    await sleep(500);

    notEqual(window.activeTextEditor, undefined);
    const uri = window.activeTextEditor!.document.uri;
    strictEqual(uri.toString(), 'output:oxc.oxc-vscode.Oxc');
  });

  // ToDo: check why this is not working,
  // even with .gitignore deleted in th test_workspace
  //
  // test('oxc.fixAll', async () => {
  //   const edit = new WorkspaceEdit();
  //   edit.createFile(fileUri, {
  //     contents: Buffer.from('/* 😊 */debugger;'),
  //   });
  //
  //   await workspace.applyEdit(edit);
  //   await window.showTextDocument(fileUri);
  //   await sleep(500);
  //   await commands.executeCommand('oxc.fixAll', {
  //     uri: fileUri.toString(),
  //   });
  //   await workspace.saveAll();
  //   await sleep(1000);
  //
  //   const content = await readFile(fileUri.fsPath, {
  //     encoding: 'utf8',
  //   });
  //
  //  strictEqual(content, '/* 😊 */');
  // });
});
