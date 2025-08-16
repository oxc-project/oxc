import { ConfigurationChangeEvent, ConfigurationTarget, workspace, WorkspaceFolder } from 'vscode';
import { ConfigService } from './ConfigService';

export const oxlintConfigFileName = '.oxlintrc.json';

export type Trigger = 'onSave' | 'onType';

type UnusedDisableDirectives = 'allow' | 'warn' | 'deny';

/**
 * See `"contributes.configuration"` in `package.json`
 */
export interface WorkspaceConfigInterface {
  /**
   * oxlint config path
   *
   * `oxc.configPath`
   *
   * @default null
   */
  configPath: string | null;
  /**
   * typescript config path
   *
   * `oxc.tsConfigPath`
   *
   * @default null
   */
  tsConfigPath: string | null;
  /**
   * When to run the linter and generate diagnostics
   * `oxc.lint.run`
   *
   * @default 'onType'
   */
  run: Trigger;

  /**
   * Define how directive comments like `// oxlint-disable-line` should be reported,
   * when no errors would have been reported on that line anyway
   *
   * `oxc.unusedDisableDirectives`
   *
   * @default 'allow'
   */
  unusedDisableDirectives: UnusedDisableDirectives;
  /**
   * Additional flags to pass to the LSP binary
   * `oxc.flags`
   *
   * @default {}
   */
  flags: Record<string, string>;
}

export class WorkspaceConfig {
  private _configPath: string | null = null;
  private _tsConfigPath: string | null = null;
  private _runTrigger: Trigger = 'onType';
  private _unusedDisableDirectives: UnusedDisableDirectives = 'allow';
  private _flags: Record<string, string> = {};

  constructor(private readonly workspace: WorkspaceFolder) {
    this.refresh();
  }

  private get configuration() {
    return workspace.getConfiguration(ConfigService.namespace, this.workspace);
  }

  public refresh(): void {
    const flags = this.configuration.get<Record<string, string>>('flags') ?? {};
    const useNestedConfigs = !('disable_nested_config' in flags);

    this._runTrigger = this.configuration.get<Trigger>('lint.run') || 'onType';
    this._configPath = this.configuration.get<string | null>('configPath') ||
      (useNestedConfigs ? null : oxlintConfigFileName);
    this._tsConfigPath = this.configuration.get<string | null>('tsConfigPath') ?? null;
    this._unusedDisableDirectives = this.configuration.get<UnusedDisableDirectives>('unusedDisableDirectives') ??
      'allow';
    this._flags = flags;
  }

  public effectsConfigChange(event: ConfigurationChangeEvent): boolean {
    if (event.affectsConfiguration(`${ConfigService.namespace}.configPath`, this.workspace)) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.tsConfigPath`, this.workspace)) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.lint.run`, this.workspace)) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.unusedDisableDirectives`, this.workspace)) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.flags`, this.workspace)) {
      return true;
    }
    return false;
  }

  public get isCustomConfigPath(): boolean {
    return this.configPath !== null && this.configPath !== oxlintConfigFileName;
  }

  get runTrigger(): Trigger {
    return this._runTrigger;
  }

  updateRunTrigger(value: Trigger): PromiseLike<void> {
    this._runTrigger = value;
    return this.configuration.update('lint.run', value, ConfigurationTarget.WorkspaceFolder);
  }

  get configPath(): string | null {
    return this._configPath;
  }

  updateConfigPath(value: string | null): PromiseLike<void> {
    this._configPath = value;
    return this.configuration.update('configPath', value, ConfigurationTarget.WorkspaceFolder);
  }

  get tsConfigPath(): string | null {
    return this._tsConfigPath;
  }

  updateTsConfigPath(value: string | null): PromiseLike<void> {
    this._tsConfigPath = value;
    return this.configuration.update('tsConfigPath', value, ConfigurationTarget.WorkspaceFolder);
  }

  get unusedDisableDirectives(): UnusedDisableDirectives {
    return this._unusedDisableDirectives;
  }

  updateUnusedDisableDirectives(value: UnusedDisableDirectives): PromiseLike<void> {
    this._unusedDisableDirectives = value;
    return this.configuration.update('unusedDisableDirectives', value, ConfigurationTarget.WorkspaceFolder);
  }

  get flags(): Record<string, string> {
    return this._flags;
  }

  updateFlags(value: Record<string, string>): PromiseLike<void> {
    this._flags = value;
    return this.configuration.update('flags', value, ConfigurationTarget.WorkspaceFolder);
  }

  public toLanguageServerConfig(): WorkspaceConfigInterface {
    return {
      run: this.runTrigger,
      configPath: this.configPath ?? null,
      tsConfigPath: this.tsConfigPath ?? null,
      unusedDisableDirectives: this.unusedDisableDirectives,
      flags: this.flags,
    };
  }
}
