import { commands, Diagnostic, extensions, languages, Uri, window, workspace, WorkspaceEdit } from 'vscode';
import path = require('path');

type OxlintConfigPlugins = string[];
type OxlintConfigCategories = Record<string, unknown>;
type OxlintConfigGlobals = Record<string, 'readonly' | 'writeable' | 'off'>;
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

export const WORKSPACE_FOLDER = workspace.workspaceFolders![0];
export const WORKSPACE_DIR = WORKSPACE_FOLDER.uri;

const rootOxlintConfigUri = Uri.joinPath(WORKSPACE_DIR, '.oxlintrc.json');

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function activateExtension(): Promise<void> {
  const ext = extensions.getExtension('oxc.oxc-vscode')!;
  if (!ext.isActive) {
    await ext.activate();
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

export async function loadFixture(fixture: string, workspaceUri: Uri = WORKSPACE_DIR): Promise<void> {
  const absolutePath = path.resolve(`${__dirname}/../fixtures/${fixture}`);
  // do not copy directly into the workspace folder. FileWatcher will detect them as a deletion and stop itself.
  await workspace.fs.copy(Uri.file(absolutePath), Uri.joinPath(workspaceUri, 'fixtures'), { overwrite: true });
}

export async function getDiagnostics(file: string, workspaceUri: Uri = WORKSPACE_DIR): Promise<Diagnostic[]> {
  const fileUri = Uri.joinPath(workspaceUri, 'fixtures', file);
  await window.showTextDocument(fileUri);
  await sleep(500);
  const diagnostics = languages.getDiagnostics(fileUri);
  await commands.executeCommand('workbench.action.closeActiveEditor');
  return diagnostics;
}
