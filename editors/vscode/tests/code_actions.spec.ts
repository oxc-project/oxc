
import { deepStrictEqual, strictEqual } from 'assert';
import {
  commands,
  Position,
  Range,
  Uri,
  window,
  workspace,
  WorkspaceEdit
} from 'vscode';
import {
  activateExtension,
  loadFixture,
  sleep,
  WORKSPACE_DIR
} from './test-helpers';
import assert = require('assert');

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

suite('code actions', () => {
  test('listed code actions', async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('debugger;'),
      overwrite: true,
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);

    const codeActions = await commands.executeCommand(
      'vscode.executeCodeActionProvider',
      fileUri,
      {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 9 },
      },
    );

    assert(Array.isArray(codeActions));
    const quickFixes = codeActions.filter(
      (action) => action.kind?.value === 'quickfix',
    );
    strictEqual(quickFixes.length, 3);
    deepStrictEqual(
      quickFixes.map(({ edit: _edit, kind: _kind, ...fix }) => ({
        ...fix,
      })),
      [
        {
          isPreferred: true,
          title: 'Remove the debugger statement',
        },
        {
          isPreferred: false,
          title: 'Disable no-debugger for this line',
        },
        {
          isPreferred: false,
          title: 'Disable no-debugger for this file',
        },
      ],
    );
  });

  // https://github.com/oxc-project/oxc/issues/10422
  test('code action `source.fixAll.oxc` on editor.codeActionsOnSave', async () => {
    let file = Uri.joinPath(WORKSPACE_DIR, 'fixtures', 'file.js');
    let expectedFile = Uri.joinPath(WORKSPACE_DIR, 'fixtures', 'expected.txt');

    await workspace.getConfiguration('editor').update('codeActionsOnSave', {
      'source.fixAll.oxc': 'always',
    });
    await workspace.saveAll();

    const range = new Range(new Position(0, 0), new Position(0, 0));
    const edit = new WorkspaceEdit();
    edit.replace(file, range, ' ');

    await sleep(1000);

    await loadFixture('fixall_with_code_actions_on_save');
    await workspace.openTextDocument(file);
    await workspace.applyEdit(edit);
    await sleep(1000);
    await workspace.saveAll();
    await sleep(500);

    const content = await workspace.fs.readFile(file);
    const expected = await workspace.fs.readFile(expectedFile);

    strictEqual(content.toString(), expected.toString());

    await workspace.getConfiguration('editor').update('codeActionsOnSave', undefined);
    await workspace.saveAll();
  });
});
