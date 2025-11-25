import { workspace } from "vscode";
import { ConfigService } from "./ConfigService";

export class VSCodeConfig implements VSCodeConfigInterface {
  private _enable!: boolean;
  private _trace!: TraceLevel;
  private _binPathOxlint: string | undefined;
  private _nodePath: string | undefined;
  private _requireConfig!: boolean;

  constructor() {
    this.refresh();
  }

  private get configuration() {
    return workspace.getConfiguration(ConfigService.namespace);
  }

  public refresh(): void {
    let binPathOxlint = this.configuration.get<string>("path.oxlint");
    // fallback to deprecated 'path.server' setting
    if (!binPathOxlint) {
      binPathOxlint = this.configuration.get<string>("path.server");
    }
    this._enable = this.configuration.get<boolean>("enable") ?? true;
    this._trace = this.configuration.get<TraceLevel>("trace.server") || "off";
    this._binPathOxlint = binPathOxlint;
    this._nodePath = this.configuration.get<string>("path.node");
    this._requireConfig = this.configuration.get<boolean>("requireConfig") ?? false;
  }

  get enable(): boolean {
    return this._enable;
  }

  updateEnable(value: boolean): PromiseLike<void> {
    this._enable = value;
    return this.configuration.update("enable", value);
  }

  get trace(): TraceLevel {
    return this._trace;
  }

  updateTrace(value: TraceLevel): PromiseLike<void> {
    this._trace = value;
    return this.configuration.update("trace.server", value);
  }

  get binPathOxlint(): string | undefined {
    return this._binPathOxlint;
  }

  updateBinPathOxlint(value: string | undefined): PromiseLike<void> {
    this._binPathOxlint = value;
    return this.configuration.update("path.oxlint", value);
  }

  get nodePath(): string | undefined {
    return this._nodePath;
  }

  updateNodePath(value: string | undefined): PromiseLike<void> {
    this._nodePath = value;
    return this.configuration.update("path.node", value);
  }

  get requireConfig(): boolean {
    return this._requireConfig;
  }

  updateRequireConfig(value: boolean): PromiseLike<void> {
    this._requireConfig = value;
    return this.configuration.update("requireConfig", value);
  }
}

type TraceLevel = "off" | "messages" | "verbose";

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
   * Path to the `oxlint` binary
   * `oxc.path.oxlint`
   * @default undefined
   */
  binPathOxlint: string | undefined;

  /**
   * Path to Node.js
   * `oxc.path.node`
   * @default undefined
   */
  nodePath: string | undefined;

  /**
   * Start the language server only when a `.oxlintrc.json` file exists in one of the workspaces.
   * `oxc.requireConfig`
   * @default false
   */
  requireConfig: boolean;
}
