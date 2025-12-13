import { Func } from 'mocha';
import { commands, Diagnostic, extensions, languages, Uri, window, workspace, WorkspaceEdit, WorkspaceFolder } from 'vscode';
import path = require('path');

type OxlintConfigPlugins = string[];
type OxlintConfigCategories = Record<string, unknown>;
type OxlintConfigGlobals = Record<string, 'readonly' | 'writable' | 'off'>;
type OxlintConfigEnv = Record<string, boolean>;
type OxlintConfigIgnorePatterns = string[];
type OxlintRuleSeverity = 'off' | 'warn' | 'error';
type OxlintConfigRules = Record<string, OxlintRuleSeverity | [OxlintRuleSeverity, Record<string, unknown>]>;

export type OxlintConfigOverride = {
  files: string[];
  env?: OxlintConfigEnv;
  globals?: OxlintConfigGlobals;
  plugins?: OxlintConfigPlugins;
  categories?: OxlintConfigCategories;
  rules?: OxlintConfigRules;
};

export type OxlintConfig = {
  $schema?: string;
  env?: OxlintConfigEnv;
  globals?: OxlintConfigGlobals;
  plugins?: OxlintConfigPlugins;
  categories?: OxlintConfigCategories;
  rules?: OxlintConfigRules;
  overrides?: OxlintConfigOverride[];
  ignorePatterns?: OxlintConfigIgnorePatterns;
};

export const WORKSPACE_FOLDER: WorkspaceFolder = workspace.workspaceFolders![0];
export const WORKSPACE_SECOND_FOLDER: WorkspaceFolder | undefined = workspace.workspaceFolders![1];

export const WORKSPACE_DIR = WORKSPACE_FOLDER.uri;
export const WORKSPACE_SECOND_DIR = WORKSPACE_SECOND_FOLDER?.uri;


const rootOxlintConfigUri = Uri.joinPath(WORKSPACE_DIR, '.oxlintrc.json');

export function testSingleFolderMode(title: string, fn: Func) {
  if (process.env['SINGLE_FOLDER_WORKSPACE'] !== 'true') {
    return;
  }

  test(`${title} (single folder workspace)`, fn);
}

export function testMultiFolderMode(title: string, fn: Func) {
  if (process.env['MULTI_FOLDER_WORKSPACE'] !== 'true') {
    return;
  }

  test(`${title} (multi folder workspace)`, fn);
}

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function activateExtension(full: boolean = true): Promise<void> {
  const ext = extensions.getExtension('oxc.oxc-vscode')!;
  if (!ext.isActive) {
    await ext.activate();
  }

  if (full) {
    await loadFixture('debugger');
    const fileUri = Uri.joinPath(fixturesWorkspaceUri(), 'fixtures', 'debugger.js');
    await window.showTextDocument(fileUri);
    // wait for initialized requests
    await sleep(250);
    await commands.executeCommand('workbench.action.closeActiveEditor');
  }
}

export async function createOxlintConfiguration(configuration: OxlintConfig): Promise<void> {
  const edit = new WorkspaceEdit();
  edit.createFile(rootOxlintConfigUri, {
    contents: Buffer.from(JSON.stringify(configuration)),
    overwrite: true,
  });
  await workspace.applyEdit(edit);
}

// in multi folder setup we want to load the fixtures into the second folder.
// the first folder should be already covered by the single folder setup.
export function fixturesWorkspaceUri(): Uri {
  if (process.env['MULTI_FOLDER_WORKSPACE'] === 'true') {
    return WORKSPACE_SECOND_DIR!;
  }

  return WORKSPACE_DIR;
}

export async function loadFixture(fixture: string, workspaceDir: Uri = fixturesWorkspaceUri()): Promise<void> {
  const absolutePath = path.resolve(`${__dirname}/../fixtures/${fixture}`);
  // do not copy directly into the workspace folder. FileWatcher will detect them as a deletion and stop itself.
  await workspace.fs.copy(Uri.file(absolutePath), Uri.joinPath(workspaceDir, 'fixtures'), { overwrite: true });
}

export async function getDiagnostics(file: string, workspaceDir: Uri = fixturesWorkspaceUri()): Promise<Diagnostic[]> {
  const diagnostics = await getDiagnosticsWithoutClose(file, workspaceDir);
  await commands.executeCommand('workbench.action.closeActiveEditor');
  return diagnostics;
}

export async function getDiagnosticsWithoutClose(file: string, workspaceDir: Uri = fixturesWorkspaceUri()): Promise<Diagnostic[]> {
  const fileUri = Uri.joinPath(workspaceDir, 'fixtures', file);
  await window.showTextDocument(fileUri);
  await sleep(250);
  const diagnostics = languages.getDiagnostics(fileUri);
  return diagnostics;
}

export async function writeToFixtureFile(file: string, content: string, workspaceDir: Uri = fixturesWorkspaceUri()): Promise<void> {
   const fileUri = Uri.joinPath(workspaceDir, 'fixtures', file);
  await window.showTextDocument(fileUri);

  for (const char of content) {
      // oxlint-disable eslint/no-await-in-loop -- simulate key presses
      await commands.executeCommand('type', { text: char });
      await sleep(50);
      // oxlint-enable eslint/no-await-in-loop
  }
}

export async function waitForDiagnosticChange(): Promise<void> {
    return new Promise<void>((resolve) =>
      languages.onDidChangeDiagnostics(() => {
        resolve();
      })
    );
}

