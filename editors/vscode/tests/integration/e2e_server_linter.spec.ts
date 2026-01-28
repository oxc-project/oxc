import { strictEqual } from "assert";
import {
  commands,
  DiagnosticSeverity,
  languages,
  Position,
  Range,
  Uri,
  window,
  workspace,
  WorkspaceEdit,
} from "vscode";
import {
  activateExtension,
  createOxlintConfiguration,
  fixturesWorkspaceUri,
  getDiagnostics,
  getDiagnosticsWithoutClose,
  loadFixture,
  sleep,
  testMultiFolderMode,
  testSingleFolderMode,
  waitForDiagnosticChange,
  WORKSPACE_DIR,
  WORKSPACE_SECOND_DIR,
  writeToFixtureFile,
} from "../test-helpers";
import assert = require("assert");

const fileUri = Uri.joinPath(WORKSPACE_DIR, "debugger.js");

suiteSetup(async () => {
  await activateExtension();
});

teardown(async () => {
  await workspace.getConfiguration("oxc").update("fixKind", undefined);
  await workspace.getConfiguration("oxc").update("tsConfigPath", undefined);
  await workspace.getConfiguration("oxc").update("typeAware", undefined);
  await workspace.getConfiguration("oxc").update("fmt.experimental", undefined);
  await workspace.getConfiguration("oxc").update("fmt.configPath", undefined);
  await workspace.getConfiguration("editor").update("defaultFormatter", undefined);
  await workspace.saveAll();
});

suite("E2E Server Linter", () => {
  // Skip tests if linter tests are disabled
  if (process.env.SKIP_LINTER_TEST === "true") {
    return;
  }

  test("simple debugger statement", async () => {
    await loadFixture("debugger");
    const diagnostics = await getDiagnostics("debugger.js");

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == "object");
    strictEqual(diagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(
      diagnostics[0].message,
      "`debugger` statement is not allowed\nhelp: Remove the debugger statement",
    );
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Warning);
    strictEqual(diagnostics[0].range.start.line, 0);
    strictEqual(diagnostics[0].range.start.character, 8);
    strictEqual(diagnostics[0].range.end.line, 0);
    strictEqual(diagnostics[0].range.end.character, 17);
  });

  for (const ext of [
    "astro",
    "cjs",
    "cts",
    "js",
    "jsx",
    "mjs",
    "mts",
    "svelte",
    "ts",
    "tsx",
    "vue",
  ]) {
    testSingleFolderMode(`file extension ${ext}`, async () => {
      await loadFixture("file_extensions");
      const diagnostics = await getDiagnostics(`debugger.${ext}`);

      strictEqual(diagnostics.length, 1);
      assert(typeof diagnostics[0].code == "object");
      strictEqual(diagnostics[0].code.target.authority, "oxc.rs");
      strictEqual(
        diagnostics[0].message,
        "`debugger` statement is not allowed\nhelp: Remove the debugger statement",
      );
      strictEqual(diagnostics[0].severity, DiagnosticSeverity.Warning);
    });
  }

  // TODO: fix flaky test, broken since 25-12-16
  // the `.oxlintrc.json` is not picked up correctly and `no-empty-file` is still enabled
  test.skip("detects diagnostics on run", async () => {
    // testSingleFolderMode
    // const config = new VSCodeConfig();
    // config.updateTrace('verbose');
    // await commands.executeCommand('oxc.showOutputChannel');

    await loadFixture("lint_on_run");
    await sleep(500);
    const diagnostics = await getDiagnosticsWithoutClose(`onType.ts`);
    // process.stdout.write(JSON.stringify(diagnostics, null, 2));

    strictEqual(diagnostics.length, 0, "no diagnostics expected initially");

    await writeToFixtureFile("onType.ts", "debugger;");
    await waitForDiagnosticChange();
    const updatedDiagnostics = await getDiagnosticsWithoutClose(`onType.ts`);
    strictEqual(updatedDiagnostics.length, 1);

    await workspace.saveAll();
    await sleep(500);

    const sameDiagnostics = await getDiagnostics(`onType.ts`);
    strictEqual(updatedDiagnostics.length, sameDiagnostics.length);
  });

  test("empty oxlint configuration behaves like default configuration", async () => {
    await loadFixture("debugger_empty_config");
    await sleep(500);
    const diagnostics = await getDiagnostics("debugger.js", undefined, 500);

    strictEqual(diagnostics.length, 1);
    assert(typeof diagnostics[0].code == "object");
    strictEqual(diagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(
      diagnostics[0].message,
      "`debugger` statement is not allowed\nhelp: Remove the debugger statement",
    );
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Warning);
    strictEqual(diagnostics[0].range.start.line, 0);
    strictEqual(diagnostics[0].range.start.character, 8);
    strictEqual(diagnostics[0].range.end.line, 0);
    strictEqual(diagnostics[0].range.end.character, 17);
  });

  test("setting rule to error, will report the diagnostic as error", async () => {
    const edit = new WorkspaceEdit();
    edit.createFile(fileUri, {
      contents: Buffer.from("/* 😊 */debugger;"),
      overwrite: true,
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await sleep(500);
    const diagnosticsWarning = languages.getDiagnostics(fileUri);

    strictEqual(diagnosticsWarning.length, 1);
    strictEqual(
      diagnosticsWarning[0].severity,
      DiagnosticSeverity.Warning,
      "Expect default severity to be warning",
    );

    await createOxlintConfiguration({
      rules: {
        "no-debugger": "error",
      },
    });
    await workspace.saveAll();
    await waitForDiagnosticChange();
    const diagnosticsError = languages.getDiagnostics(fileUri);

    strictEqual(diagnosticsError.length, 1);
    strictEqual(
      diagnosticsError[0].severity,
      DiagnosticSeverity.Error,
      "Expect changed severity to be error",
    );
  });

  // We can check the changed with kind with `vscode.executeCodeActionProvider`
  // but to be safe that everything works, we will check the applied changes.
  // This way we can be sure that everything works as expected.
  test("auto detect changing `fixKind` with fixAll command", async () => {
    const originalContent = "if (x === -0) { bar();}";
    await createOxlintConfiguration({
      rules: {
        "no-compare-neg-zero": "error",
      },
    });

    const edit = new WorkspaceEdit();

    edit.createFile(fileUri, {
      contents: Buffer.from(originalContent),
      overwrite: true,
    });

    await workspace.applyEdit(edit);
    await window.showTextDocument(fileUri);
    await commands.executeCommand("oxc.fixAll", {
      uri: fileUri.toString(),
    });
    await workspace.saveAll();

    const content = await workspace.fs.readFile(fileUri);

    strictEqual(content.toString(), originalContent);
    await workspace.getConfiguration("oxc").update("fixKind", "safe_fix_or_suggestion");
    // wait for server to update the internal linter
    await sleep(500);
    await workspace.saveAll();

    await commands.executeCommand("oxc.fixAll", {
      uri: fileUri.toString(),
    });
    await sleep(500);
    await workspace.saveAll();
    const contentWithFixAll = await workspace.fs.readFile(fileUri);

    strictEqual(contentWithFixAll.toString(), "if (Object.is(x, -0)) { bar();}");
  });

  test("nested configs severity", async () => {
    await loadFixture("nested_config");
    const rootDiagnostics = await getDiagnostics("index.ts", undefined, 500);
    const nestedDiagnostics = await getDiagnostics("folder/index.ts", undefined, 500);

    assert(typeof rootDiagnostics[0].code == "object");
    strictEqual(rootDiagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(rootDiagnostics[0].severity, DiagnosticSeverity.Warning);

    assert(typeof nestedDiagnostics[0].code == "object");
    strictEqual(nestedDiagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(nestedDiagnostics[0].severity, DiagnosticSeverity.Error);
  });

  testMultiFolderMode("different diagnostic severity", async () => {
    await loadFixture("debugger", WORKSPACE_DIR);
    await loadFixture("debugger_error", WORKSPACE_SECOND_DIR);
    await sleep(500);

    const firstDiagnostics = await getDiagnostics("debugger.js", WORKSPACE_DIR);
    const secondDiagnostics = await getDiagnostics("debugger.js", WORKSPACE_SECOND_DIR);

    assert(typeof firstDiagnostics[0].code == "object");
    strictEqual(firstDiagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(firstDiagnostics[0].severity, DiagnosticSeverity.Warning);

    assert(typeof secondDiagnostics[0].code == "object");
    strictEqual(secondDiagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(secondDiagnostics[0].severity, DiagnosticSeverity.Error);
  });

  // somehow this test is flaky in CI
  test.skip("changing config from `extends` will revalidate the diagnostics", async () => {
    await loadFixture("changing_extended_config");
    const firstDiagnostics = await getDiagnostics("debugger.js");

    assert(typeof firstDiagnostics[0].code == "object");
    strictEqual(firstDiagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(firstDiagnostics[0].severity, DiagnosticSeverity.Warning);

    const oxlintExtendedConfigUri = Uri.joinPath(
      fixturesWorkspaceUri(),
      "fixtures",
      "folder/custom.json",
    );
    const oxlintExtendedConfig = JSON.stringify({
      rules: {
        "no-debugger": "error",
      },
    });

    const edit = new WorkspaceEdit();
    edit.replace(
      oxlintExtendedConfigUri,
      new Range(new Position(0, 0), new Position(100, 100)),
      oxlintExtendedConfig,
    );

    await window.showTextDocument(oxlintExtendedConfigUri);
    await workspace.applyEdit(edit);
    await workspace.saveAll();
    await waitForDiagnosticChange();
    const secondDiagnostics = await getDiagnostics("debugger.js");

    assert(typeof secondDiagnostics[0].code == "object");
    strictEqual(secondDiagnostics[0].code.target.authority, "oxc.rs");
    strictEqual(secondDiagnostics[0].severity, DiagnosticSeverity.Error);
  });

  testSingleFolderMode("changing oxc.tsConfigPath will revalidate the diagnostics", async () => {
    await loadFixture("changing_tsconfig_path");
    const firstDiagnostics = await getDiagnostics("deep/src/dep-a.ts");

    strictEqual(firstDiagnostics.length, 0);

    await workspace.getConfiguration("oxc").update("tsConfigPath", "fixtures/deep/tsconfig.json");
    await workspace.saveAll();
    await waitForDiagnosticChange();

    const secondDiagnostics = await getDiagnostics("deep/src/dep-a.ts");
    strictEqual(secondDiagnostics.length, 1);
    assert(typeof secondDiagnostics[0].code == "object");
    strictEqual(secondDiagnostics[0].code.target.authority, "oxc.rs");
    assert(secondDiagnostics[0].message.startsWith("Dependency cycle detected"));
    strictEqual(secondDiagnostics[0].severity, DiagnosticSeverity.Error);
  });

  // flaky because of https://github.com/oxc-project/tsgolint/issues/349
  test.skip("changing oxc.typeAware will revalidate the tsgolint diagnostics", async () => {
    await loadFixture("type_aware");
    await sleep(500); // wait for server to pick up the new config
    const firstDiagnostics = await getDiagnosticsWithoutClose("index.ts");

    await workspace.getConfiguration("oxc").update("fixKind", "all");
    strictEqual(firstDiagnostics.length, 0);

    await workspace.getConfiguration("oxc").update("typeAware", true);
    await workspace.saveAll();
    await waitForDiagnosticChange();

    const secondDiagnostics = await getDiagnostics("index.ts");
    strictEqual(secondDiagnostics.length, 1);
  });

  testSingleFolderMode("changing oxc.enable will update the client status", async () => {
    await loadFixture("changing_enable");

    const firstDiagnostics = await getDiagnostics("debugger.js");
    strictEqual(firstDiagnostics.length, 1);

    await workspace.getConfiguration("oxc").update("enable", false);
    await workspace.saveAll();
    await waitForDiagnosticChange();

    const secondDiagnostics = await getDiagnostics("debugger.js");
    strictEqual(secondDiagnostics.length, 0);

    // enable it for other tests
    await workspace.getConfiguration("oxc").update("enable", true);
    await workspace.saveAll();
    await sleep(500);
  });

  test("js plugin support", async () => {
    await loadFixture("js_plugins");
    await sleep(500);

    const diagnostics = await getDiagnosticsWithoutClose("index.js");
    strictEqual(diagnostics.length, 1);

    assert(typeof diagnostics[0].code == "string");
    strictEqual(diagnostics[0].code, "js-plugin(test-rule)");
    strictEqual(diagnostics[0].message, "Custom name JS Plugin Test Rule.");
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Error);
  });
});
