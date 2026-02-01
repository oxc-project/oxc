import { strictEqual } from "assert";
import { DiagnosticSeverity } from "vscode";
import {
  activateExtension,
  deleteFixtures,
  getDiagnosticsWithoutClose,
  loadFixture,
  sleep,
} from "../test-helpers";
import assert = require("assert");

suiteSetup(async () => {
  await activateExtension();
});

teardown(async () => {
  await deleteFixtures();
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
    await sleep(500);

    const diagnostics = await getDiagnosticsWithoutClose("index.js");
    strictEqual(diagnostics.length, 1);

    assert(typeof diagnostics[0].code == "string");
    strictEqual(diagnostics[0].code, "js-plugin(test-rule)");
    strictEqual(diagnostics[0].message, "Custom name JS Plugin Test Rule.");
    strictEqual(diagnostics[0].severity, DiagnosticSeverity.Error);
  });
});
