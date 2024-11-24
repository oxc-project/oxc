import { workspace } from 'vscode';

export class Config implements ConfigInterface {
  private static readonly _namespace = 'oxc';

  private _runTrigger!: Trigger;
  private _enable!: boolean;
  private _trace!: TraceLevel;
  private _configPath!: string;
  private _binPath: string | undefined;

  constructor() {
    this.refresh();
  }

  public refresh(): void {
    const conf = workspace.getConfiguration(Config._namespace);

    this._runTrigger = conf.get<Trigger>('lint.run') || 'onType';
    this._enable = conf.get<boolean>('enable') ?? true;
    this._trace = conf.get<TraceLevel>('trace.server') || 'off';
    this._configPath = conf.get<string>('configPath') || '.oxlintrc.json';
    this._binPath = conf.get<string>('path.server');
  }

  get runTrigger(): Trigger {
    return this._runTrigger;
  }

  updateRunTrigger(value: Trigger): PromiseLike<void> {
    this._runTrigger = value;
    return workspace
      .getConfiguration(Config._namespace)
      .update('lint.run', value);
  }

  get enable(): boolean {
    return this._enable;
  }

  updateEnable(value: boolean): PromiseLike<void> {
    this._enable = value;
    return workspace
      .getConfiguration(Config._namespace)
      .update('enable', value);
  }

  get trace(): TraceLevel {
    return this._trace;
  }

  updateTrace(value: TraceLevel): PromiseLike<void> {
    this._trace = value;
    return workspace
      .getConfiguration(Config._namespace)
      .update('trace.server', value);
  }

  get configPath(): string {
    return this._configPath;
  }

  updateConfigPath(value: string): PromiseLike<void> {
    this._configPath = value;
    return workspace
      .getConfiguration(Config._namespace)
      .update('configPath', value);
  }

  get binPath(): string | undefined {
    return this._binPath;
  }

  updateBinPath(value: string | undefined): PromiseLike<void> {
    this._binPath = value;
    return workspace
      .getConfiguration(Config._namespace)
      .update('path.server', value);
  }

  public toLanguageServerConfig(): LanguageServerConfig {
    return {
      run: this.runTrigger,
      enable: this.enable,
      configPath: this.configPath,
    };
  }
}

interface LanguageServerConfig {
  configPath: string;
  enable: boolean;
  run: Trigger;
}

export type Trigger = 'onSave' | 'onType';
type TraceLevel = 'off' | 'messages' | 'verbose';
/**
 * See `"contributes.configuration"` in `package.json`
 */
interface ConfigInterface {
  /**
   * When to run the linter and generate diagnostics
   * `oxc.lint.run`
   *
   * @default 'onType'
   */
  runTrigger: Trigger;
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
   * oxlint config path
   *
   * `oxc.configPath`
   *
   * @default ".oxlintrc.json"
   */
  configPath: string;
  /**
   * Path to LSP binary
   * `oxc.path.server`
   * @default undefined
   */
  binPath: string | undefined;
}
