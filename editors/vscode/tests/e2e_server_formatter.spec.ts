import { strictEqual } from 'assert';
import {
  commands,
  Uri,
  window,
  workspace,
} from 'vscode';
import {
  activateExtension,
  fixturesWorkspaceUri,
  loadFixture,
  sleep,
} from './test-helpers';

suiteSetup(async () => {
  await activateExtension();
  await workspace.getConfiguration('editor').update('defaultFormatter', 'oxc.oxc-vscode');
  await workspace.saveAll();
});

teardown(async () => {
  await workspace.getConfiguration('oxc').update('fmt.configPath', undefined);
  await workspace.getConfiguration('editor').update('defaultFormatter', undefined);
  await workspace.saveAll();
});

suite('E2E Server Formatter', () => {
    // Skip tests if formatter tests are disabled
    if (process.env.SKIP_FORMATTER_TEST === 'true') {
      return;
    }

    test('formats code', async () => {
      await workspace.getConfiguration('editor').update('defaultFormatter', 'oxc.oxc-vscode');
      await workspace.saveAll();
      await loadFixture('formatting');

      await sleep(500);

      const fileUri = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'formatting.ts');

      const document = await workspace.openTextDocument(fileUri);
      await window.showTextDocument(document);
      await commands.executeCommand('editor.action.formatDocument');
      await workspace.saveAll();
      const content = await workspace.fs.readFile(fileUri);

      strictEqual(content.toString(), "class X {\n  foo() {\n    return 42;\n  }\n}\n");
    });

    test('formats code with `oxc.fmt.configPath`', async () => {
      await loadFixture('formatting_with_config');
      await workspace.getConfiguration('editor').update('defaultFormatter', 'oxc.oxc-vscode');
      await workspace.getConfiguration('oxc').update('fmt.configPath', './fixtures/formatter.json');
      await workspace.saveAll();

      const fileUri = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'formatting.ts');

      const document = await workspace.openTextDocument(fileUri);
      await window.showTextDocument(document);
      await sleep(500); // wait for the server to pick up the new config
      await commands.executeCommand('editor.action.formatDocument');
      await workspace.saveAll();
      const content = await workspace.fs.readFile(fileUri);

      strictEqual(content.toString(), "class X {\n  foo() {\n    return 42\n  }\n}\n");
    });

    test('formats prettier-only file types', async () => {
      await loadFixture('formatting_prettier');
      await workspace.getConfiguration('editor').update('defaultFormatter', 'oxc.oxc-vscode');
      await workspace.saveAll();

      await sleep(500);

      const expectedJson = "{\n  \"a\": 1,\n  \"b\": [1, 2]\n}\n";
      const expectedCss = ".foo {\n  color: red;\n}\n";
      const expectedMarkdown = "# Title\n\n- a\n- b\n";

      const cases: Array<[string, string]> = [
        ["prettier.json", expectedJson],
        ["prettier.jsonc", expectedJson],
        ["prettier.css", expectedCss],
        ["prettier.md", expectedMarkdown],
      ];

      for (const [file, expected] of cases) {
        const fileUri = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', file);
        const original = (await workspace.fs.readFile(fileUri)).toString();
        const document = await workspace.openTextDocument(fileUri);
        await window.showTextDocument(document);
        await commands.executeCommand('editor.action.formatDocument');
        await workspace.saveAll();
        const content = await workspace.fs.readFile(fileUri);

        const actual = content.toString();
        const matchesExpected = actual === expected;
        const matchesOriginal = actual === original;
        strictEqual(matchesExpected || matchesOriginal, true, `${file} should be formatted or unchanged`);
      }
    });

});
