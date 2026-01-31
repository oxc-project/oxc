import { strictEqual } from "assert";
import { DiagnosticSeverity, languages, Uri, window, workspace } from "vscode";
import { activateExtension, fixturesWorkspaceUri, loadFixture, sleep } from "../test-helpers";
import assert = require("assert");

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

  test("js plugin support", async () => {
    // Flaky test; JS plugin support is a work in progress.
    // this test is allowed to fail in CI until then.
    // Make sure to run the test in CI multiple times before marking it as fixed.
    // Move the test into `e2e_server_linter.spec.ts` once stable.
    if (process.env.OXLINT_JS_PLUGIN !== "true") {
      return;
    }
    await loadFixture("js_plugins");

    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "index.js");
    await window.showTextDocument(fileUri);

    // Wait for diagnostics to appear with retry logic.
    // JS plugins may take longer to initialize than native linting.
    let diagnostics = languages.getDiagnostics(fileUri);
    for (let i = 0; i < 20 && diagnostics.length === 0; i++) {
      await sleep(250);
      diagnostics = languages.getDiagnostics(fileUri);
    }

    strictEqual(diagnostics.length, 1);

    assert(typeof diagnostics[0].code == "string");
    strictEqual(diagnostics[0].code, "js-plugin(test-rule)");
    strictEqual(diagnostics[0].message, "Custom name JS Plugin Test Rule.");
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Error);
  });
});
