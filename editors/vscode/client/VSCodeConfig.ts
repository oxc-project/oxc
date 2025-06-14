import { workspace } from 'vscode';
import { ConfigService } from './ConfigService';

export class VSCodeConfig implements VSCodeConfigInterface {
  private _enable!: boolean;
  private _trace!: TraceLevel;
  private _binPath: string | undefined;
  private _requireConfig!: boolean;

  constructor() {
    this.refresh();
  }

  private get configuration() {
    return workspace.getConfiguration(ConfigService.namespace);
  }

  public refresh(): void {
    this._enable = this.configuration.get<boolean>('enable') ?? true;
    this._trace = this.configuration.get<TraceLevel>('trace.server') || 'off';
    this._binPath = this.configuration.get<string>('path.server');
    this._requireConfig = this.configuration.get<boolean>('requireConfig') ?? false;
  }

  get enable(): boolean {
    return this._enable;
  }

  updateEnable(value: boolean): PromiseLike<void> {
    this._enable = value;
    return this.configuration.update('enable', value);
  }

  get trace(): TraceLevel {
    return this._trace;
  }

  updateTrace(value: TraceLevel): PromiseLike<void> {
    this._trace = value;
    return this.configuration.update('trace.server', value);
  }

  get binPath(): string | undefined {
    return this._binPath;
  }

  updateBinPath(value: string | undefined): PromiseLike<void> {
    this._binPath = value;
    return this.configuration.update('path.server', value);
  }

  get requireConfig(): boolean {
    return this._requireConfig;
  }

  updateRequireConfig(value: boolean): PromiseLike<void> {
    this._requireConfig = value;
    return this.configuration.update('requireConfig', value);
  }
}

type TraceLevel = 'off' | 'messages' | 'verbose';

/**
 * See `"contributes.configuration"` in `package.json`
 */
interface VSCodeConfigInterface {
  /**
   * `oxc.enable`
   *
   * @default true
   */
  enable: boolean;
  /**
   * Trace VSCode <-> Oxc Language Server communication
   * `oxc.trace.server`
   *
   * @default 'off'
   */
  trace: TraceLevel;
  /**
   * Path to LSP binary
   * `oxc.path.server`
   * @default undefined
   */
  binPath: string | undefined;
  /**
   * Start the language server only when a `.oxlintrc.json` file exists in one of the workspaces.
   * `oxc.requireConfig`
   * @default false
   */
  requireConfig: boolean;
}
