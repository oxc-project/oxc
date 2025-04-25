import { workspace, WorkspaceConfiguration } from 'vscode';
import { ConfigService } from './ConfigService';
import { oxlintConfigFileName } from './VSCodeConfig';

export type Trigger = 'onSave' | 'onType';

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
   * When to run the linter and generate diagnostics
   * `oxc.lint.run`
   *
   * @default 'onType'
   */
  run: Trigger;
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
  private _runTrigger: Trigger = 'onType';
  private _flags: Record<string, string> = {};

  constructor(configuration: WorkspaceConfiguration) {
    this.refresh(configuration);
  }

  public refresh(configuration: WorkspaceConfiguration): void {
    const flags = configuration.get<Record<string, string>>('flags') ?? {};
    const useNestedConfigs = !('disable_nested_config' in flags);

    this._runTrigger = configuration.get<Trigger>('lint.run') || 'onType';
    this._configPath = configuration.get<string | null>('configPath') ||
      (useNestedConfigs ? null : oxlintConfigFileName);
    this._flags = flags;
  }

  get runTrigger(): Trigger {
    return this._runTrigger;
  }

  updateRunTrigger(value: Trigger): PromiseLike<void> {
    this._runTrigger = value;
    return workspace
      .getConfiguration(ConfigService.namespace)
      .update('lint.run', value);
  }

  get configPath(): string | null {
    return this._configPath;
  }

  updateConfigPath(value: string): PromiseLike<void> {
    this._configPath = value;
    return workspace
      .getConfiguration(ConfigService.namespace)
      .update('configPath', value);
  }

  get flags(): Record<string, string> {
    return this._flags;
  }

  updateFlags(value: Record<string, string>): PromiseLike<void> {
    this._flags = value;
    return workspace
      .getConfiguration(ConfigService.namespace)
      .update('flags', value);
  }

  public toLanguageServerConfig(): WorkspaceConfigInterface {
    return {
      run: this.runTrigger,
      configPath: this.configPath ?? null,
      flags: this.flags,
    };
  }
}
