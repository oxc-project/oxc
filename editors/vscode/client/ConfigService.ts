import { ConfigurationChangeEvent, workspace } from 'vscode';
import { IDisposable } from './types';
import { VSCodeConfig } from './VSCodeConfig';
import { WorkspaceConfig } from './WorkspaceConfig';

export class ConfigService implements IDisposable {
  public static readonly namespace = 'oxc';
  private readonly _disposables: IDisposable[] = [];

  public vsCodeConfig: VSCodeConfig;

  private _workspaceConfig: WorkspaceConfig;

  public onConfigChange:
    | ((this: ConfigService, config: ConfigurationChangeEvent) => Promise<void>)
    | undefined;

  constructor() {
    const conf = workspace.getConfiguration(ConfigService.namespace);
    this.vsCodeConfig = new VSCodeConfig(conf);
    this._workspaceConfig = new WorkspaceConfig(conf);
    this.onConfigChange = undefined;

    const disposeChangeListener = workspace.onDidChangeConfiguration(
      this.onVscodeConfigChange.bind(this),
    );
    this._disposables.push(disposeChangeListener);
  }

  public get rootServerConfig(): WorkspaceConfig {
    return this._workspaceConfig;
  }

  public refresh(): void {
    const conf = workspace.getConfiguration(ConfigService.namespace);
    this.vsCodeConfig.refresh(conf);
    this.rootServerConfig.refresh(conf);
  }

  private async onVscodeConfigChange(event: ConfigurationChangeEvent): Promise<void> {
    if (event.affectsConfiguration(ConfigService.namespace)) {
      this.refresh();
      await this.onConfigChange?.(event);
    }
  }

  dispose() {
    for (const disposable of this._disposables) {
      disposable.dispose();
    }
  }
}
