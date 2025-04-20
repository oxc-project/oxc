import { ConfigurationChangeEvent, Uri, workspace, WorkspaceFolder } from 'vscode';
import { IDisposable } from './types';
import { VSCodeConfig } from './VSCodeConfig';
import { oxlintConfigFileName, WorkspaceConfig, WorkspaceConfigInterface } from './WorkspaceConfig';

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

  // ToDo: refactor it so we do not rely on this function
  // this is only used for the initialize options on the server which is a custom object between the client and server
  // we can change it to support multiple workspaces and avoid sending `workspace/configuration` request to the client.
  // Make sure to still support the old way on server side for other clients.
  public get rootLanguageServerConfig(): WorkspaceConfigInterface {
    // will be true when the user uses `code file.ts` in cli
    if (workspace.workspaceFolders === undefined) {
      // fallback to the default config
      return {
        configPath: null,
        run: 'onType',
        flags: {},
      };
    }

    return this.workspaceConfigs.get(workspace.workspaceFolders[0].uri.path)!.toLanguageServerConfig();
  }

  public addWorkspaceConfig(workspace: WorkspaceFolder): WorkspaceConfig {
    let workspaceConfig = new WorkspaceConfig(workspace);
    this.workspaceConfigs.set(workspace.uri.path, workspaceConfig);
    return workspaceConfig;
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

  public effectsWorkspaceConfigPathChange(event: ConfigurationChangeEvent): boolean {
    for (const workspaceConfig of this.workspaceConfigs.values()) {
      if (workspaceConfig.effectsConfigPathChange(event)) {
        return true;
      }
    }
    return false;
  }

  public getOxlintCustomConfigs(): string[] {
    const customConfigs: string[] = [];
    for (const [path, config] of this.workspaceConfigs.entries()) {
      if (config.configPath && config.configPath !== oxlintConfigFileName) {
        customConfigs.push(`${path}/${config.configPath}`);
      }
    }
    return customConfigs;
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
