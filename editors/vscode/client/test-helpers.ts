import { extensions, workspace } from 'vscode';

export const WORKSPACE_DIR = workspace.workspaceFolders![0].uri.toString();

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function activateExtension(): Promise<void> {
  const ext = extensions.getExtension('oxc.oxc-vscode')!;
  if (!ext.isActive) {
    await ext.activate();
  }
}
