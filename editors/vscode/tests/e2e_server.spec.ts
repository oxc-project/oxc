import { strictEqual } from 'assert';
import {
  commands,
  DiagnosticSeverity,
  languages,
  Uri,
  window,
  workspace,
  WorkspaceEdit
} from 'vscode';
import {
  activateExtension,
  createOxlintConfiguration,
  getDiagnostics,
  loadFixture,
  sleep,
  testMultiFolderMode,
  WORKSPACE_DIR,
  WORKSPACE_SECOND_DIR
} from './test-helpers';
import assert = require('assert');

const fileUri = Uri.joinPath(WORKSPACE_DIR, 'debugger.js');

suiteSetup(async () => {
  await activateExtension();
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

  test('nested configs severity', async () => {
    await loadFixture('nested_config');
    const rootDiagnostics = await getDiagnostics('index.ts');
    const nestedDiagnostics = await getDiagnostics('folder/index.ts');

    assert(typeof rootDiagnostics[0].code == 'object');
    strictEqual(rootDiagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(rootDiagnostics[0].severity, DiagnosticSeverity.Warning);

    assert(typeof nestedDiagnostics[0].code == 'object');
    strictEqual(nestedDiagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(nestedDiagnostics[0].severity, DiagnosticSeverity.Error);
  });

  testMultiFolderMode('different diagnostic severity', async () => {
    await loadFixture('debugger');
    await loadFixture('debugger_error', WORKSPACE_SECOND_DIR);

    const firstDiagnostics = await getDiagnostics('debugger.js');
    const secondDiagnostics = await getDiagnostics('debugger.js', WORKSPACE_SECOND_DIR);

    assert(typeof firstDiagnostics[0].code == 'object');
    strictEqual(firstDiagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(firstDiagnostics[0].severity, DiagnosticSeverity.Warning);

    assert(typeof secondDiagnostics[0].code == 'object');
    strictEqual(secondDiagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(secondDiagnostics[0].severity, DiagnosticSeverity.Error);
  })

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
});
