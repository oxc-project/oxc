import { deepStrictEqual, notEqual, strictEqual } from 'assert';
import { readFile } from 'fs/promises';
import {
  CodeAction,
  commands,
  DiagnosticSeverity,
  languages,
  ProviderResult,
  Uri,
  window,
  workspace,
  WorkspaceEdit,
} from 'vscode';
import { activateExtension, sleep, WORKSPACE_DIR } from './test-helpers';
import assert = require('assert');

const filePath = WORKSPACE_DIR + '/debugger.js';
const fileUri = Uri.parse(filePath);

suite('commands', () => {
  setup(async () => {
    await activateExtension();
  });

  suiteTeardown(async () => {
    const edit = new WorkspaceEdit();
    edit.deleteFile(fileUri, {
      ignoreIfNotExists: true,
    });
    await workspace.applyEdit(edit);
  });

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

  test('oxc.toggleEnable', async () => {
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
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('/* 😊 */debugger;'),
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await commands.executeCommand('oxc.fixAll', {
      uri: fileUri.toString(),
    });
    await workspace.saveAll();

    const content = await readFile(fileUri.fsPath, {
      encoding: 'utf8',
    });

    strictEqual(content, '/* 😊 */');
  });
});

suite('E2E Diagnostics', () => {
  setup(async () => {
    await activateExtension();
  });

  suiteTeardown(async () => {
    const edit = new WorkspaceEdit();
    edit.deleteFile(fileUri, {
      ignoreIfNotExists: true,
    });
    await workspace.applyEdit(edit);
  });

  test('simple debugger statement', async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('/* 😊 */debugger;'),
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    const diagnostics = languages.getDiagnostics(fileUri);

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == 'object');
    strictEqual(diagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(diagnostics[0].message, '`debugger` statement is not allowed\nhelp: Delete this code.');
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Warning);
    strictEqual(diagnostics[0].range.start.line, 0);
    strictEqual(diagnostics[0].range.start.character, 8);
    strictEqual(diagnostics[0].range.end.line, 0);
    strictEqual(diagnostics[0].range.end.character, 17);
  });

  test('code actions', async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('debugger;'),
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);

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
          title: 'Fix this `debugger` statement is not allowed\nhelp problem',
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
});
