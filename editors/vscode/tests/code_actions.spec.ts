
import { deepStrictEqual, strictEqual } from 'assert';
import {
  CodeAction,
  commands,
  Position,
  ProviderResult,
  Range,
  Uri,
  window,
  workspace,
  WorkspaceEdit
} from 'vscode';
import {
  activateExtension,
  fixturesWorkspaceUri,
  loadFixture,
  sleep
} from './test-helpers';
import assert = require('assert');

setup(async () => {
  await activateExtension();
});

suite('code actions', () => {
  test('listed code actions', async () => {
    await loadFixture('debugger');
    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'debugger.js');
    // await window.showTextDocument(fileUri); -- should also work without opening the file

    const codeActions: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
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
    let file = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'file.js');
    let expectedFile = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'expected.txt');

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

  test('changing configuration "fix_kind" will reveal more code actions', async () => {
    await loadFixture('changing_fix_kind');
    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'for_direction.ts');
    await window.showTextDocument(fileUri);
    const codeActionsNoFix: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      'vscode.executeCodeActionProvider',
      fileUri,
      {
        start: { line: 0, character: 18 },
        end: { line: 0, character: 19 },
      },
    );

    assert(Array.isArray(codeActionsNoFix));
    const quickFixesNoFix = codeActionsNoFix.filter(
      (action) => action.kind?.value === 'quickfix',
    );
    strictEqual(quickFixesNoFix.length, 2);

    await workspace.getConfiguration('oxc').update('flags', {
      'fix_kind': 'dangerous_fix',
    });
    await workspace.saveAll();

    const codeActionsWithFix: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      'vscode.executeCodeActionProvider',
      fileUri,
      {
        start: { line: 0, character: 18 },
        end: { line: 0, character: 19 },
      },
    );

    assert(Array.isArray(codeActionsWithFix));
    const quickFixesWithFix = codeActionsWithFix.filter(
      (action) => action.kind?.value === 'quickfix',
    );
    strictEqual(quickFixesWithFix.length, 3);
  });
});
