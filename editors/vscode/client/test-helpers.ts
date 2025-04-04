import { extensions, Uri, workspace, WorkspaceEdit } from 'vscode';

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

export const WORKSPACE_DIR = workspace.workspaceFolders![0].uri.toString();

const rootOxlintConfigPath = WORKSPACE_DIR + '/.oxlintrc.json';
const rootOxlintConfigUri = Uri.parse(rootOxlintConfigPath);

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

export async function deleteOxlintConfiguration(): Promise<void> {
  const edit = new WorkspaceEdit();
  edit.deleteFile(rootOxlintConfigUri, {
    ignoreIfNotExists: true,
  });
  await workspace.applyEdit(edit);
  await sleep(500);
}
