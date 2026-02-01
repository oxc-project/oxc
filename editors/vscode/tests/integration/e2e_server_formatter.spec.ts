import { strictEqual } from "assert";
import { commands, Uri, window, workspace } from "vscode";
import {
  activateExtension,
  deleteFixtures,
  fixturesWorkspaceUri,
  loadFixture,
  sleep,
} from "../test-helpers";

suiteSetup(async () => {
  await activateExtension();
  await workspace.getConfiguration("editor").update("defaultFormatter", "oxc.oxc-vscode");
  await workspace.saveAll();
});

teardown(async () => {
  await workspace.getConfiguration("oxc").update("fmt.configPath", undefined);
  await workspace.getConfiguration("editor").update("defaultFormatter", undefined);
  await workspace.saveAll();
  await deleteFixtures();
});

suite("E2E Server Formatter", () => {
  // Skip tests if formatter tests are disabled
  if (process.env.SKIP_FORMATTER_TEST === "true") {
    return;
  }

  test("formats code", async () => {
    await workspace.getConfiguration("editor").update("defaultFormatter", "oxc.oxc-vscode");
    await workspace.saveAll();
    await loadFixture("formatting");

    await sleep(500);

    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "formatting.ts");

    const document = await workspace.openTextDocument(fileUri);
    await window.showTextDocument(document);
    await commands.executeCommand("editor.action.formatDocument");
    await workspace.saveAll();
    const content = await workspace.fs.readFile(fileUri);

    strictEqual(content.toString(), "class X {\n  foo() {\n    return 42;\n  }\n}\n");
  });

  test("formats code with `oxc.fmt.configPath`", async () => {
    await loadFixture("formatting_with_config");
    await workspace.getConfiguration("editor").update("defaultFormatter", "oxc.oxc-vscode");
    await workspace.getConfiguration("oxc").update("fmt.configPath", "./fixtures/formatter.json");
    await workspace.saveAll();

    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), "fixtures", "formatting.ts");

    const document = await workspace.openTextDocument(fileUri);
    await window.showTextDocument(document);
    await sleep(500);
    await commands.executeCommand("editor.action.formatDocument");
    await workspace.saveAll();
    const content = await workspace.fs.readFile(fileUri);

    strictEqual(content.toString(), "class X {\n  foo() {\n    return 42\n  }\n}\n");
  });
});
