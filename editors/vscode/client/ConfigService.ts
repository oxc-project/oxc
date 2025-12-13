import * as path from "node:path";
import { ConfigurationChangeEvent, RelativePattern, Uri, workspace, WorkspaceFolder } from "vscode";
import { validateSafeBinaryPath } from "./PathValidator";
import { IDisposable } from "./types";
import { VSCodeConfig } from "./VSCodeConfig";
import {
  OxfmtWorkspaceConfigInterface,
  OxlintWorkspaceConfigInterface,
  WorkspaceConfig,
  WorkspaceConfigInterface,
} from "./WorkspaceConfig";

export class ConfigService implements IDisposable {
  public static readonly namespace = "oxc";
  private readonly _disposables: IDisposable[] = [];

  /**
   * Indicates whether the `oxc_language_server` is being used as the formatter.
   * If true, the formatter functionality is handled by the language server itself.
   */
  public useOxcLanguageServerForFormatting: boolean = false;

  public vsCodeConfig: VSCodeConfig;

  private workspaceConfigs: Map<string, WorkspaceConfig> = new Map();

  public onConfigChange:
    | ((this: ConfigService, config: ConfigurationChangeEvent) => Promise<void>)
    | undefined;

  constructor() {
    this.vsCodeConfig = new VSCodeConfig();
    const { workspaceFolders } = workspace;
    if (workspaceFolders) {
      for (const folder of workspaceFolders) {
        this.addWorkspaceConfig(folder);
      }
    }
    this.onConfigChange = undefined;

    const disposeChangeListener = workspace.onDidChangeConfiguration(
      this.onVscodeConfigChange.bind(this),
    );
    this._disposables.push(disposeChangeListener);
  }

  public get languageServerConfig(): {
    workspaceUri: string;
    options: WorkspaceConfigInterface | OxlintWorkspaceConfigInterface;
  }[] {
    return [...this.workspaceConfigs.entries()].map(([path, config]) => {
      const options = this.useOxcLanguageServerForFormatting
        ? config.toLanguageServerConfig()
        : config.toOxlintConfig();

      return {
        workspaceUri: Uri.file(path).toString(),
        options,
      };
    });
  }

  public get formatterServerConfig(): {
    workspaceUri: string;
    options: OxfmtWorkspaceConfigInterface;
  }[] {
    return [...this.workspaceConfigs.entries()].map(([path, config]) => ({
      workspaceUri: Uri.file(path).toString(),
      options: config.toOxfmtConfig(),
    }));
  }

  public addWorkspaceConfig(workspace: WorkspaceFolder): void {
    this.workspaceConfigs.set(workspace.uri.path, new WorkspaceConfig(workspace));
  }

  public removeWorkspaceConfig(workspace: WorkspaceFolder): void {
    this.workspaceConfigs.delete(workspace.uri.path);
  }

  public getWorkspaceConfig(workspace: Uri): WorkspaceConfig | undefined {
    return this.workspaceConfigs.get(workspace.path);
  }

  public effectsWorkspaceConfigChange(event: ConfigurationChangeEvent): boolean {
    for (const workspaceConfig of this.workspaceConfigs.values()) {
      if (workspaceConfig.effectsConfigChange(event)) {
        return true;
      }
    }
    return false;
  }

  public getUserServerBinPath(): string | undefined {
    let bin = this.vsCodeConfig.binPathOxlint;
    if (!bin) {
      return;
    }

    // validates the given path is safe to use
    if (validateSafeBinaryPath(bin) === false) {
      return;
    }

    if (!path.isAbsolute(bin)) {
      // if the path is not absolute, resolve it to the first workspace folder
      const cwd = this.workspaceConfigs.keys().next().value;
      if (!cwd) {
        return;
      }
      bin = path.normalize(path.join(cwd, bin));
      // strip the leading slash on Windows
      if (process.platform === "win32" && bin.startsWith("\\")) {
        bin = bin.slice(1);
      }
    }

    return bin;
  }

  public async getOxfmtServerBinPath(): Promise<string | undefined> {
    let bin = this.vsCodeConfig.binPathOxfmt;

    const cwd = this.workspaceConfigs.keys().next().value;
    if (!cwd) {
      return undefined;
    }

    if (!bin) {
      // try to find oxfmt in node_modules/.bin, resolve to the first workspace folder
      const files = await workspace.findFiles(
        new RelativePattern(cwd, "**/node_modules/.bin/oxfmt"),
        null,
        1,
      );

      return files.length > 0 ? files[0].fsPath : undefined;
    }

    // validates the given path is safe to use
    if (validateSafeBinaryPath(bin) === false) {
      return undefined;
    }

    if (!path.isAbsolute(bin)) {
      // if the path is not absolute, resolve it to the first workspace folder
      bin = path.normalize(path.join(cwd, bin));
      // strip the leading slash on Windows
      if (process.platform === "win32" && bin.startsWith("\\")) {
        bin = bin.slice(1);
      }
    }

    return bin;
  }

  private async onVscodeConfigChange(event: ConfigurationChangeEvent): Promise<void> {
    let isConfigChanged = false;

    if (event.affectsConfiguration(ConfigService.namespace)) {
      this.vsCodeConfig.refresh();
      isConfigChanged = true;
    }

    for (const workspaceConfig of this.workspaceConfigs.values()) {
      if (workspaceConfig.effectsConfigChange(event)) {
        workspaceConfig.refresh();
        isConfigChanged = true;
      }
    }

    if (isConfigChanged) {
      await this.onConfigChange?.(event);
    }
  }

  dispose() {
    for (const disposable of this._disposables) {
      void disposable.dispose();
    }
  }
}
