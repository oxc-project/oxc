import { deepStrictEqual, notEqual, strictEqual } from 'assert';
import {
  CodeAction,
  commands,
  DiagnosticSeverity,
  languages,
  Position,
  ProviderResult,
  Range,
  Uri,
  window,
  workspace,
  WorkspaceEdit,
} from 'vscode';
import {
  activateExtension,
  createOxlintConfiguration,
  getDiagnostics,
  loadFixture,
  sleep,
  WORKSPACE_DIR,
} from './test-helpers';
import assert = require('assert');

const fileUri = Uri.joinPath(WORKSPACE_DIR, 'debugger.js');

setup(async () => {
  await activateExtension();
});

suite('commands', () => {
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

suite('code actions', () => {
  test('listed code actions', async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('debugger;'),
      overwrite: true,
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

suite('E2E Diagnostics', () => {
  test('simple debugger statement', async () => {
    await loadFixture('debugger');
    const diagnostics = await getDiagnostics('debugger.js');

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == 'object');
    strictEqual(diagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(diagnostics[0].message, '`debugger` statement is not allowed\nhelp: Remove the debugger statement');
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Warning);
    strictEqual(diagnostics[0].range.start.line, 0);
    strictEqual(diagnostics[0].range.start.character, 8);
    strictEqual(diagnostics[0].range.end.line, 0);
    strictEqual(diagnostics[0].range.end.character, 17);
  });

  test('empty oxlint configuration behaves like default configuration', async () => {
    await loadFixture('debugger_empty_config');
    const diagnostics = await getDiagnostics('debugger.js');

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == 'object');
    strictEqual(diagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(diagnostics[0].message, '`debugger` statement is not allowed\nhelp: Remove the debugger statement');
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Warning);
    strictEqual(diagnostics[0].range.start.line, 0);
    strictEqual(diagnostics[0].range.start.character, 8);
    strictEqual(diagnostics[0].range.end.line, 0);
    strictEqual(diagnostics[0].range.end.character, 17);
  });

  test('cross module', async () => {
    await loadFixture('cross_module');
    const diagnostics = await getDiagnostics('dep-a.ts');

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == 'object');
    strictEqual(diagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(
      diagnostics[0].message,
      'Dependency cycle detected\nhelp: These paths form a cycle: \n-> ./dep-b.ts - fixtures/dep-b.ts\n-> ./dep-a.ts - fixtures/dep-a.ts',
    );
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Error);
    strictEqual(diagnostics[0].range.start.line, 1);
    strictEqual(diagnostics[0].range.start.character, 18);
    strictEqual(diagnostics[0].range.end.line, 1);
    strictEqual(diagnostics[0].range.end.character, 30);
  });

  test('cross module with nested config', async () => {
    await loadFixture('cross_module_nested_config');
    const diagnostics = await getDiagnostics('folder/folder-dep-a.ts');

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == 'object');
    strictEqual(diagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(
      diagnostics[0].message,
      'Dependency cycle detected\nhelp: These paths form a cycle: \n-> ./folder-dep-b.ts - fixtures/folder/folder-dep-b.ts\n-> ./folder-dep-a.ts - fixtures/folder/folder-dep-a.ts',
    );
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Error);
    strictEqual(diagnostics[0].range.start.line, 1);
    strictEqual(diagnostics[0].range.start.character, 18);
    strictEqual(diagnostics[0].range.end.line, 1);
    strictEqual(diagnostics[0].range.end.character, 37);
  });

  test('cross module with extended config', async () => {
    await loadFixture('cross_module_extended_config');
    const diagnostics = await getDiagnostics('dep-a.ts');

    assert(typeof diagnostics[0].code == 'object');
    strictEqual(diagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(
      diagnostics[0].message,
      'Dependency cycle detected\nhelp: These paths form a cycle: \n-> ./dep-b.ts - fixtures/dep-b.ts\n-> ./dep-a.ts - fixtures/dep-a.ts',
    );
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Error);
    strictEqual(diagnostics[0].range.start.line, 1);
    strictEqual(diagnostics[0].range.start.character, 18);
    strictEqual(diagnostics[0].range.end.line, 1);
    strictEqual(diagnostics[0].range.end.character, 30);
  });

  test('setting rule to error, will report the diagnostic as error', async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from('/* 😊 */debugger;'),
      overwrite: true,
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await sleep(500);
    const diagnosticsWarning = languages.getDiagnostics(fileUri);

    strictEqual(diagnosticsWarning.length, 1);
    strictEqual(diagnosticsWarning[0].severity, DiagnosticSeverity.Warning, 'Expect default severity to be warning');

    const resolve = new Promise<void>((resolve) =>
      languages.onDidChangeDiagnostics(() => {
        const diagnosticsError = languages.getDiagnostics(fileUri);

        strictEqual(diagnosticsError.length, 1);
        strictEqual(diagnosticsError[0].severity, DiagnosticSeverity.Error, 'Expect changed severity to be error');
        resolve();
      })
    );

    await createOxlintConfiguration({
      rules: {
        'no-debugger': 'error',
      },
    });
    await workspace.saveAll();
    await resolve;
  });

  // We can check the changed with kind with `vscode.executeCodeActionProvider`
  // but to be safe that everything works, we will check the applied changes.
  // This way we can be sure that everything works as expected.
  test('auto detect changing `fix_kind` flag with fixAll command', async () => {
    const originalContent = 'if (foo == null) { bar();}';

    await createOxlintConfiguration({
      rules: {
        'no-eq-null': 'error',
      },
    });

    const edit = new WorkspaceEdit();

    edit.createFile(fileUri, {
      contents: Buffer.from(originalContent),
      overwrite: true,
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await commands.executeCommand('oxc.fixAll', {
      uri: fileUri.toString(),
    });
    await workspace.saveAll();

    const content = await workspace.fs.readFile(fileUri);

    strictEqual(content.toString(), originalContent);
    await workspace.getConfiguration('oxc').update('flags', {
      fix_kind: 'all',
    });
    // wait for server to update the internal linter
    await sleep(500);
    await workspace.saveAll();

    await commands.executeCommand('oxc.fixAll', {
      uri: fileUri.toString(),
    });
    await sleep(500);
    await workspace.saveAll();
    const contentWithFixAll = await workspace.fs.readFile(fileUri);

    strictEqual(contentWithFixAll.toString(), 'if (foo === null) { bar();}');
  });
});
