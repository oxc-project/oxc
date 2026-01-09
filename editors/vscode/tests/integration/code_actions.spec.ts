import { deepStrictEqual, strictEqual } from "assert";
import {
  CodeAction,
  commands,
  ConfigurationTarget,
  Position,
  ProviderResult,
  Range,
  Uri,
  window,
  workspace,
  WorkspaceEdit,
} from "vscode";
import {
  activateExtension,
  fixturesWorkspaceUri,
  loadFixture,
  sleep,
  testSingleFolderMode,
} from "../test-helpers";
import assert = require("assert");

suiteSetup(async () => {
  await activateExtension();
});

teardown(async () => {
  const vsConfig = workspace.getConfiguration("oxc");
  const wsConfig = workspace.getConfiguration("oxc", fixturesWorkspaceUri());
  await vsConfig.update("unusedDisableDirectives", undefined);
  await wsConfig.update("fixKind", undefined, ConfigurationTarget.WorkspaceFolder);
  await workspace.getConfiguration("editor").update("codeActionsOnSave", undefined);
  await workspace.saveAll();
});

suite("code actions", () => {
  // Skip tests if linter tests are disabled
  if (process.env.SKIP_LINTER_TEST === "true") {
    return;
  }

  // flaky test for multi workspace mode
  testSingleFolderMode("listed code actions", async () => {
    await loadFixture("debugger_empty_config");
    await sleep(500);
    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "debugger.js");
    // await window.showTextDocument(fileUri); -- should also work without opening the file

    const codeActions: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      "vscode.executeCodeActionProvider",
      fileUri,
      {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 9 },
      },
    );

    assert(Array.isArray(codeActions));
    const quickFixes = codeActions.filter((action) => action.kind?.value === "quickfix");
    deepStrictEqual(
      quickFixes.map(({ edit: _edit, kind: _kind, ...fix }) => {
        return fix;
      }),
      [
        {
          isPreferred: true,
          title: "Remove the debugger statement",
        },
        {
          isPreferred: false,
          title: "Disable no-debugger for this line",
        },
        {
          isPreferred: false,
          title: "Disable no-debugger for this whole file",
        },
      ],
    );
  });

  // https://github.com/oxc-project/oxc/issues/10422
  test("code action `source.fixAll.oxc` on editor.codeActionsOnSave", async () => {
    const file = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "file.js");
    const expectedFile = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "expected.txt");

    await workspace.getConfiguration("editor").update("codeActionsOnSave", {
      "source.fixAll.oxc": "always",
    });
    await workspace.saveAll();

    const range = new Range(new Position(0, 0), new Position(0, 0));
    const edit = new WorkspaceEdit();
    edit.replace(file, range, " ");

    await sleep(1000);

    await loadFixture("fixall_with_code_actions_on_save");
    await workspace.openTextDocument(file);
    await workspace.applyEdit(edit);
    await sleep(1000);
    await workspace.saveAll();
    await sleep(500);

    const content = await workspace.fs.readFile(file);
    const expected = await workspace.fs.readFile(expectedFile);

    strictEqual(content.toString(), expected.toString());
  });

  // https://discord.com/channels/1079625926024900739/1080723403595591700/1422191300395929620
  test('code action `source.fixAll.oxc` ignores "ignore this rule for this line/file"', async () => {
    const file = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "file2.js");
    const expectedFile = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "expected.txt");

    await workspace.getConfiguration("editor").update("codeActionsOnSave", {
      "source.fixAll.oxc": "always",
    });
    await workspace.saveAll();

    const range = new Range(new Position(0, 0), new Position(0, 0));
    const edit = new WorkspaceEdit();
    edit.replace(file, range, " ");

    await sleep(1000);

    await loadFixture("fixall_code_action_ignore_only_disable_fix");
    await workspace.openTextDocument(file);
    await workspace.applyEdit(edit);
    await sleep(1000);
    await workspace.saveAll();
    await sleep(500);

    const content = await workspace.fs.readFile(file);
    const expected = await workspace.fs.readFile(expectedFile);

    strictEqual(content.toString(), expected.toString());
  });

  test('changing configuration flag "fix_kind" will reveal more code actions', async () => {
    await loadFixture("changing_fix_kind");
    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "for_direction.ts");
    await window.showTextDocument(fileUri);
    const codeActionsNoFix: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      "vscode.executeCodeActionProvider",
      fileUri,
      {
        start: { line: 0, character: 18 },
        end: { line: 0, character: 19 },
      },
    );

    assert(Array.isArray(codeActionsNoFix));
    const quickFixesNoFix = codeActionsNoFix.filter((action) => action.kind?.value === "quickfix");
    strictEqual(quickFixesNoFix.length, 2);

    await workspace
      .getConfiguration("oxc", fixturesWorkspaceUri())
      .update("fixKind", "dangerous_fix", ConfigurationTarget.WorkspaceFolder);
    await workspace.saveAll();

    const codeActionsWithFix: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      "vscode.executeCodeActionProvider",
      fileUri,
      {
        start: { line: 0, character: 18 },
        end: { line: 0, character: 19 },
      },
    );

    assert(Array.isArray(codeActionsWithFix));
    const quickFixesWithFix = codeActionsWithFix.filter(
      (action) => action.kind?.value === "quickfix",
    );
    strictEqual(quickFixesWithFix.length, 3);
  });

  test('changing configuration "unusedDisableDirectives" will reveal more code actions', async () => {
    await loadFixture("changing_unused_disable_directives");
    const fileUri = Uri.joinPath(
      fixturesWorkspaceUri(),
      "fixtures",
      "unused_disable_directives.js",
    );
    await window.showTextDocument(fileUri);
    const codeActionsNoFix: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      "vscode.executeCodeActionProvider",
      fileUri,
      {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 10 },
      },
    );
    assert(Array.isArray(codeActionsNoFix));
    const quickFixesNoFix = codeActionsNoFix.filter((action) => action.kind?.value === "quickfix");
    strictEqual(quickFixesNoFix.length, 0);
    await workspace.getConfiguration("oxc").update("unusedDisableDirectives", "warn");
    await workspace.saveAll();
    const codeActionsWithFix: ProviderResult<Array<CodeAction>> = await commands.executeCommand(
      "vscode.executeCodeActionProvider",
      fileUri,
      {
        start: { line: 0, character: 2 },
        end: { line: 0, character: 10 },
      },
    );
    assert(Array.isArray(codeActionsWithFix));
    const quickFixesWithFix = codeActionsWithFix.filter(
      (action) => action.kind?.value === "quickfix",
    );
    strictEqual(quickFixesWithFix.length, 1);
  });
});
