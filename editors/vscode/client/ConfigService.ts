import { ConfigurationChangeEvent, workspace } from 'vscode';
import { Config } from './Config';
import { IDisposable } from './types';

export class ConfigService implements IDisposable {
  private static readonly _namespace = 'oxc';
  private readonly _disposables: IDisposable[] = [];

  public config: Config;

  public onConfigChange:
    | ((this: ConfigService, config: ConfigurationChangeEvent) => void)
    | undefined;

  constructor() {
    this.config = new Config();
    this.onConfigChange = undefined;

    const disposeChangeListener = workspace.onDidChangeConfiguration(
      this.onVscodeConfigChange.bind(this),
    );
    this._disposables.push(disposeChangeListener);
  }

  private onVscodeConfigChange(event: ConfigurationChangeEvent): void {
    if (event.affectsConfiguration(ConfigService._namespace)) {
      this.config.refresh();
      this.onConfigChange?.call(this, event);
    }
  }

  dispose() {
    for (const disposable of this._disposables) {
      disposable.dispose();
    }
  }
}
