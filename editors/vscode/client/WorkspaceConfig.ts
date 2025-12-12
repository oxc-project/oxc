import { ConfigurationChangeEvent, ConfigurationTarget, workspace, WorkspaceFolder } from "vscode";
import { ConfigService } from "./ConfigService";

export const oxlintConfigFileName = ".oxlintrc.json";

export type Trigger = "onSave" | "onType";

type UnusedDisableDirectives = "allow" | "warn" | "deny";

export enum FixKind {
  SafeFix = "safe_fix",
  SafeFixOrSuggestion = "safe_fix_or_suggestion",
  DangerousFix = "dangerous_fix",
  DangerousFixOrSuggestion = "dangerous_fix_or_suggestion",
  None = "none",
  All = "all",
}

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
   * Whether to enable type-aware linting
   *
   * `oxc.typeAware`
   *
   * @default false
   */
  typeAware: boolean;

  /**
   * Disable nested config files detection
   * `oxc.disableNestedConfig`
   * @default false
   */
  disableNestedConfig: boolean;

  /**
   * Fix kind to use when applying fixes
   * `oxc.fixKind`
   * @default 'safe_fix'
   */
  fixKind: FixKind;

  /**
   * Additional flags to pass to the LSP binary
   * `oxc.flags`
   *
   * @default {}
   */
  flags: Record<string, string>;

  /**
   * Enable formatting experiment
   * `oxc.fmt.experimental`
   *
   * @default false
   */
  ["fmt.experimental"]: boolean;

  /**
   * Path to an oxfmt configuration file
   * `oxc.fmt.configPath`
   */
  ["fmt.configPath"]?: string | null;
}

export type OxlintWorkspaceConfigInterface = Omit<
  WorkspaceConfigInterface,
  "fmt.experimental" | "fmt.configPath"
>;

export type OxfmtWorkspaceConfigInterface = Pick<
  WorkspaceConfigInterface,
  "fmt.experimental" | "fmt.configPath"
>;

export class WorkspaceConfig {
  private _configPath: string | null = null;
  private _tsConfigPath: string | null = null;
  private _runTrigger: Trigger = "onType";
  private _unusedDisableDirectives: UnusedDisableDirectives = "allow";
  private _typeAware: boolean = false;
  private _disableNestedConfig: boolean = false;
  private _fixKind: FixKind = FixKind.SafeFix;
  private _formattingExperimental: boolean = false;
  private _formattingConfigPath: string | null = null;

  constructor(private readonly workspace: WorkspaceFolder) {
    this.refresh();
  }

  private get configuration() {
    return workspace.getConfiguration(ConfigService.namespace, this.workspace);
  }

  public refresh(): void {
    const flags = this.configuration.get<Record<string, string>>("flags") ?? {};

    // `configuration.get` takes the default value from the package.json, which is always `safe_fix`.
    // We need to check the deprecated flags.fix_kind for the real default value.
    let fixKind = this.configuration.get<FixKind>("fixKind");
    if (
      fixKind === FixKind.SafeFix &&
      flags.fix_kind !== undefined &&
      flags.fix_kind !== "safe_fix"
    ) {
      fixKind = flags.fix_kind as FixKind;
    }

    // the same for disabledNestedConfig
    let disableNestedConfig = this.configuration.get<boolean>("disableNestedConfig");
    if (disableNestedConfig === false && flags.disable_nested_config === "true") {
      disableNestedConfig = true;
    }

    this._runTrigger = this.configuration.get<Trigger>("lint.run") || "onType";
    this._configPath = this.configuration.get<string | null>("configPath") ?? null;
    this._tsConfigPath = this.configuration.get<string | null>("tsConfigPath") ?? null;
    this._unusedDisableDirectives =
      this.configuration.get<UnusedDisableDirectives>("unusedDisableDirectives") ?? "allow";
    this._typeAware = this.configuration.get<boolean>("typeAware") ?? false;
    this._disableNestedConfig = disableNestedConfig ?? false;
    this._fixKind = fixKind ?? FixKind.SafeFix;
    this._formattingExperimental = this.configuration.get<boolean>("fmt.experimental") ?? false;
    this._formattingConfigPath = this.configuration.get<string | null>("fmt.configPath") ?? null;
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
    if (
      event.affectsConfiguration(
        `${ConfigService.namespace}.unusedDisableDirectives`,
        this.workspace,
      )
    ) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.typeAware`, this.workspace)) {
      return true;
    }
    if (
      event.affectsConfiguration(`${ConfigService.namespace}.disableNestedConfig`, this.workspace)
    ) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.fixKind`, this.workspace)) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.fmt.experimental`, this.workspace)) {
      return true;
    }
    if (event.affectsConfiguration(`${ConfigService.namespace}.fmt.configPath`, this.workspace)) {
      return true;
    }
    // deprecated settings in flags
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
    return this.configuration.update("lint.run", value, ConfigurationTarget.WorkspaceFolder);
  }

  get configPath(): string | null {
    return this._configPath;
  }

  updateConfigPath(value: string | null): PromiseLike<void> {
    this._configPath = value;
    return this.configuration.update("configPath", value, ConfigurationTarget.WorkspaceFolder);
  }

  get tsConfigPath(): string | null {
    return this._tsConfigPath;
  }

  updateTsConfigPath(value: string | null): PromiseLike<void> {
    this._tsConfigPath = value;
    return this.configuration.update("tsConfigPath", value, ConfigurationTarget.WorkspaceFolder);
  }

  get unusedDisableDirectives(): UnusedDisableDirectives {
    return this._unusedDisableDirectives;
  }

  updateUnusedDisableDirectives(value: UnusedDisableDirectives): PromiseLike<void> {
    this._unusedDisableDirectives = value;
    return this.configuration.update(
      "unusedDisableDirectives",
      value,
      ConfigurationTarget.WorkspaceFolder,
    );
  }

  get typeAware(): boolean {
    return this._typeAware;
  }

  updateTypeAware(value: boolean): PromiseLike<void> {
    this._typeAware = value;
    return this.configuration.update("typeAware", value, ConfigurationTarget.WorkspaceFolder);
  }

  get disableNestedConfig(): boolean {
    return this._disableNestedConfig;
  }

  updateDisableNestedConfig(value: boolean): PromiseLike<void> {
    this._disableNestedConfig = value;
    return this.configuration.update(
      "disableNestedConfig",
      value,
      ConfigurationTarget.WorkspaceFolder,
    );
  }

  get fixKind(): FixKind {
    return this._fixKind;
  }

  updateFixKind(value: FixKind): PromiseLike<void> {
    this._fixKind = value;
    return this.configuration.update("fixKind", value, ConfigurationTarget.WorkspaceFolder);
  }

  get formattingExperimental(): boolean {
    return this._formattingExperimental;
  }

  updateFormattingExperimental(value: boolean): PromiseLike<void> {
    this._formattingExperimental = value;
    return this.configuration.update(
      "fmt.experimental",
      value,
      ConfigurationTarget.WorkspaceFolder,
    );
  }

  get formattingConfigPath(): string | null {
    return this._formattingConfigPath;
  }

  updateFormattingConfigPath(value: string | null): PromiseLike<void> {
    this._formattingConfigPath = value;
    return this.configuration.update("fmt.configPath", value, ConfigurationTarget.WorkspaceFolder);
  }

  public toLanguageServerConfig(): WorkspaceConfigInterface {
    return {
      ...this.toOxlintConfig(),
      ...this.toOxfmtConfig(),
    };
  }

  public toOxlintConfig(): OxlintWorkspaceConfigInterface {
    return {
      run: this.runTrigger,
      configPath: this.configPath ?? null,
      tsConfigPath: this.tsConfigPath ?? null,
      unusedDisableDirectives: this.unusedDisableDirectives,
      typeAware: this.typeAware,
      disableNestedConfig: this.disableNestedConfig,
      fixKind: this.fixKind,
      // deprecated, kept for backward compatibility
      flags: {
        disable_nested_config: this.disableNestedConfig ? "true" : "false",
        ...(this.fixKind ? { fix_kind: this.fixKind } : {}),
      },
    };
  }

  public toOxfmtConfig(): OxfmtWorkspaceConfigInterface {
    return {
      ["fmt.experimental"]: this.formattingExperimental,
      ["fmt.configPath"]: this.formattingConfigPath ?? null,
    };
  }
}
