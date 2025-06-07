import { strictEqual } from "assert";
import { DiagnosticSeverity, Uri, workspace } from "vscode";
import { activateExtension, getDiagnostics, loadFixture, sleep, testMultiFolderMode, WORKSPACE_DIR } from "./test-helpers";
import assert = require("assert");

suiteSetup(async () => {
  await activateExtension();
});

const FIXTURES_URI = Uri.joinPath(WORKSPACE_DIR, '..', 'fixtures');

suite('Workspace Folders', () => {
  testMultiFolderMode('shows diagnostics to newly adding folder', async () => {
    await loadFixture('debugger');
    const folderDiagnostics = await getDiagnostics('debugger.js');

    assert(typeof folderDiagnostics[0].code == 'object');
    strictEqual(folderDiagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(folderDiagnostics[0].severity, DiagnosticSeverity.Warning);

    workspace.updateWorkspaceFolders(workspace.workspaceFolders?.length ?? 0, 0, {
      name: 'fixtures',
      uri: FIXTURES_URI
    });

    await sleep(500);
    const thirdWorkspaceDiagnostics = await getDiagnostics('debugger/debugger.js', Uri.joinPath(FIXTURES_URI, '..'));

    assert(typeof thirdWorkspaceDiagnostics[0].code == 'object');
    strictEqual(thirdWorkspaceDiagnostics[0].code.target.authority, 'oxc.rs');
    strictEqual(thirdWorkspaceDiagnostics[0].severity, DiagnosticSeverity.Warning);

    // remove the workspace folder
    workspace.updateWorkspaceFolders(workspace.workspaceFolders?.length ?? 0, 1);
  })
})
