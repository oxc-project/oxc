import { strictEqual } from 'assert';
import { readFile } from 'fs/promises';
import { commands, extensions, Uri, window, workspace, WorkspaceEdit } from 'vscode';

const WORKSPACE_DIR = workspace.workspaceFolders![0].uri.toString();
const filePath = WORKSPACE_DIR + '/debugger.js';
const fileUri = Uri.parse(filePath);

async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

suite('commands', () => {
  setup(async () => {
    try {
      const ext = extensions.getExtension('oxc.oxc-vscode')!;
      await ext.activate();
      await sleep(1000);
    } catch (e) {
      console.error(e);
    }
  });

  suiteTeardown(async () => {
    const edit = new WorkspaceEdit();
    edit.deleteFile(fileUri);
    await workspace.applyEdit(edit);
  });

  test('oxc.fixAll', async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('/* 😊 */debugger;'),
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await sleep(1000);
    await commands.executeCommand('oxc.fixAll', {
      uri: fileUri.toString(),
    });

    const content = await readFile(fileUri.toString().replace(/^file:\/\//, ''), {
      encoding: 'utf8',
    });

    strictEqual(content, '/* 😊 */');
  });
});
