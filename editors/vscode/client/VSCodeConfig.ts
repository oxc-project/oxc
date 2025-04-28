import { workspace, WorkspaceConfiguration } from 'vscode';
import { ConfigService } from './ConfigService';

export const oxlintConfigFileName = '.oxlintrc.json';

export class VSCodeConfig implements VSCodeConfigInterface {
  private _enable!: boolean;
  private _trace!: TraceLevel;
  private _binPath: string | undefined;

  constructor(configuration: WorkspaceConfiguration) {
    this.refresh(configuration);
  }

  public refresh(configuration: WorkspaceConfiguration): void {
    this._enable = configuration.get<boolean>('enable') ?? true;
    this._trace = configuration.get<TraceLevel>('trace.server') || 'off';
    this._binPath = configuration.get<string>('path.server');
  }

  get enable(): boolean {
    return this._enable;
  }

  updateEnable(value: boolean): PromiseLike<void> {
    this._enable = value;
    return workspace
      .getConfiguration(ConfigService.namespace)
      .update('enable', value);
  }

  get trace(): TraceLevel {
    return this._trace;
  }

  updateTrace(value: TraceLevel): PromiseLike<void> {
    this._trace = value;
    return workspace
      .getConfiguration(ConfigService.namespace)
      .update('trace.server', value);
  }

  get binPath(): string | undefined {
    return this._binPath;
  }

  updateBinPath(value: string | undefined): PromiseLike<void> {
    this._binPath = value;
    return workspace
      .getConfiguration(ConfigService.namespace)
      .update('path.server', value);
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
