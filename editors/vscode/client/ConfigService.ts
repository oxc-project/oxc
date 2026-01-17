import * as path from "node:path";
import {
  CancellationTokenSource,
  ConfigurationChangeEvent,
  Uri,
  workspace,
  WorkspaceFolder,
} from "vscode";
import { DiagnosticPullMode } from "vscode-languageclient";
import { validateSafeBinaryPath } from "./PathValidator";
import { IDisposable } from "./types";
import { VSCodeConfig } from "./VSCodeConfig";
import {
  OxfmtWorkspaceConfigInterface,
  OxlintWorkspaceConfigInterface,
  WorkspaceConfig,
} from "./WorkspaceConfig";

export class ConfigService implements IDisposable {
  public static readonly namespace = "oxc";
  private readonly _disposables: IDisposable[] = [];

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

  public get oxlintServerConfig(): {
    workspaceUri: string;
    options: OxlintWorkspaceConfigInterface;
  }[] {
    return [...this.workspaceConfigs.entries()].map(([path, config]) => {
      const options = config.toOxlintConfig();

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

  public async getOxlintServerBinPath(): Promise<string | undefined> {
    return this.searchBinaryPath(this.vsCodeConfig.binPathOxlint, "oxlint");
  }

  public async getOxfmtServerBinPath(): Promise<string | undefined> {
    return this.searchBinaryPath(this.vsCodeConfig.binPathOxfmt, "oxfmt");
  }

  public shouldRequestDiagnostics(
    textDocumentUri: Uri,
    diagnosticPullMode: DiagnosticPullMode,
  ): boolean {
    if (!this.vsCodeConfig.enable) {
      return false;
    }

    const textDocumentPath = textDocumentUri.path;

    for (const [workspaceUri, workspaceConfig] of this.workspaceConfigs.entries()) {
      if (textDocumentPath.startsWith(workspaceUri)) {
        return workspaceConfig.shouldRequestDiagnostics(diagnosticPullMode);
      }
    }
    return false;
  }

  private async searchBinaryPath(
    settingsBinary: string | undefined,
    defaultBinaryName: string,
  ): Promise<string | undefined> {
    if (!settingsBinary) {
      return this.searchNodeModulesBin(defaultBinaryName);
    }

    if (!workspace.isTrusted) {
      return;
    }

    // validates the given path is safe to use
    if (!validateSafeBinaryPath(settingsBinary)) {
      return undefined;
    }

    if (!path.isAbsolute(settingsBinary)) {
      const cwd = this.workspaceConfigs.keys().next().value;
      if (!cwd) {
        return undefined;
      }
      // if the path is not absolute, resolve it to the first workspace folder
      settingsBinary = path.normalize(path.join(cwd, settingsBinary));
      settingsBinary = this.removeWindowsLeadingSlash(settingsBinary);
    }

    if (process.platform !== "win32" && settingsBinary.endsWith(".exe")) {
      // on non-Windows, remove `.exe` extension if present
      settingsBinary = settingsBinary.slice(0, -4);
    }

    try {
      await workspace.fs.stat(Uri.file(settingsBinary));
      return settingsBinary;
    } catch {}

    // on Windows, also check for `.exe` extension (bun uses `.exe` for its binaries)
    if (process.platform === "win32") {
      if (!settingsBinary.endsWith(".exe")) {
        settingsBinary += ".exe";
      }

      try {
        await workspace.fs.stat(Uri.file(settingsBinary));
        return settingsBinary;
      } catch {}
    }

    // no valid binary found
    return undefined;
  }

  /**
   * strip the leading slash on Windows
   */
  private removeWindowsLeadingSlash(path: string): string {
    if (process.platform === "win32" && path.startsWith("\\")) {
      return path.slice(1);
    }
    return path;
  }

  /**
   * Search for the binary in all workspaces' node_modules/.bin directories.
   * If multiple workspaces contain the binary, the first one found is returned.
   */
  private async searchNodeModulesBin(binaryName: string): Promise<string | undefined> {
    const cts = new CancellationTokenSource();
    setTimeout(() => cts.cancel(), 20000); // cancel after 20 seconds

    try {
      // search workspace root plus up to 3 subdirectory levels for the binary path
      let patterns = [
        `node_modules/.bin/${binaryName}`,
        `*/node_modules/.bin/${binaryName}`,
        `*/*/node_modules/.bin/${binaryName}`,
        `*/*/*/node_modules/.bin/${binaryName}`,
      ];

      if (process.platform === "win32") {
        // bun package manager uses `.exe` extension on Windows
        // search for both with and without `.exe` extension
        patterns = patterns.flatMap((pattern) => [`${pattern}`, `${pattern}.exe`]);
      }

      for (const pattern of patterns) {
        // maybe use `tinyglobby` later for better performance, VSCode can be slow on globbing large projects.
        // oxlint-disable-next-line no-await-in-loop -- search sequentially up the directories
        const files = await workspace.findFiles(pattern, null, 1, cts.token);
        if (files.length > 0) {
          return files[0].fsPath;
        }
      }
    } catch {}
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
