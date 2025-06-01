import { workspace } from 'vscode';
import { ConfigService } from './ConfigService';

export class VSCodeConfig implements VSCodeConfigInterface {
  private _enable!: boolean;
  private _trace!: TraceLevel;
  private _binPath: string | undefined;

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
}
