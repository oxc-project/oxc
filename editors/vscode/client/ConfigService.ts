import { ConfigurationChangeEvent, Uri, workspace, WorkspaceFolder } from 'vscode';
import { IDisposable } from './types';
import { VSCodeConfig } from './VSCodeConfig';
import { WorkspaceConfig, WorkspaceConfigInterface } from './WorkspaceConfig';

export class ConfigService implements IDisposable {
  public static readonly namespace = 'oxc';
  private readonly _disposables: IDisposable[] = [];

  public vsCodeConfig: VSCodeConfig;

  private workspaceConfigs: Map<string, WorkspaceConfig> = new Map();

  public onConfigChange:
    | ((this: ConfigService, config: ConfigurationChangeEvent) => Promise<void>)
    | undefined;

  constructor() {
    this.vsCodeConfig = new VSCodeConfig();
    const workspaceFolders = workspace.workspaceFolders;
    if (workspaceFolders) {
      for (const folder of workspaceFolders) {
        this.addWorkspaceConfig(folder);
      }
    }
    this.onConfigChange = undefined;

    const disposeChangeListener = workspace.onDidChangeConfiguration(
      this.onVscodeConfigChange.bind(this),
    );
    this._disposables.push(disposeChangeListener);
  }

  public get languageServerConfig(): { workspaceUri: string; options: WorkspaceConfigInterface }[] {
    return [...this.workspaceConfigs.entries()].map(([path, config]) => ({
      workspaceUri: Uri.file(path).toString(),
      options: config.toLanguageServerConfig(),
    }));
  }

  public addWorkspaceConfig(workspace: WorkspaceFolder): void {
    this.workspaceConfigs.set(workspace.uri.path, new WorkspaceConfig(workspace));
  }

  public removeWorkspaceConfig(workspace: WorkspaceFolder): void {
    this.workspaceConfigs.delete(workspace.uri.path);
  }

  public getWorkspaceConfig(workspace: Uri): WorkspaceConfig | undefined {
    return this.workspaceConfigs.get(workspace.path);
  }

  public effectsWorkspaceConfigChange(event: ConfigurationChangeEvent): boolean {
    for (const workspaceConfig of this.workspaceConfigs.values()) {
      if (workspaceConfig.effectsConfigChange(event)) {
        return true;
      }
    }
    return false;
  }

  private async onVscodeConfigChange(event: ConfigurationChangeEvent): Promise<void> {
    let isConfigChanged = false;

    if (event.affectsConfiguration(ConfigService.namespace)) {
      this.vsCodeConfig.refresh();
      isConfigChanged = true;
    }

    for (const workspaceConfig of this.workspaceConfigs.values()) {
      if (workspaceConfig.effectsConfigChange(event)) {
        workspaceConfig.refresh();
        isConfigChanged = true;
      }
    }

    if (isConfigChanged) {
      await this.onConfigChange?.(event);
    }
  }

  dispose() {
    for (const disposable of this._disposables) {
      disposable.dispose();
    }
  }
}
