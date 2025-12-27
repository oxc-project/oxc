import * as path from "node:path";
import { ConfigurationChangeEvent, RelativePattern, Uri, workspace, WorkspaceFolder } from "vscode";
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
    const cwd = this.workspaceConfigs.keys().next().value;
    if (!cwd) {
      return undefined;
    }

    if (!settingsBinary) {
      return this.searchNodeModulesBin(cwd, defaultBinaryName);
    }

    if (!workspace.isTrusted) {
      return;
    }

    // validates the given path is safe to use
    if (!validateSafeBinaryPath(settingsBinary)) {
      return undefined;
    }

    if (!path.isAbsolute(settingsBinary)) {
      // if the path is not absolute, resolve it to the first workspace folder
      settingsBinary = path.normalize(path.join(cwd, settingsBinary));
      settingsBinary = this.removeWindowsLeadingSlash(settingsBinary);
    }

    return settingsBinary;
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
   * Search for the binary in the workspace's node_modules/.bin directory.
   */
  private async searchNodeModulesBin(
    workspacePath: string,
    binaryName: string,
  ): Promise<string | undefined> {
    // try to find the binary in workspace's node_modules/.bin.
    //
    // Performance: this is a fast check before searching with glob.
    // glob on windows is very slow.
    const binPath = this.removeWindowsLeadingSlash(
      path.normalize(path.join(workspacePath, "node_modules", ".bin", binaryName)),
    );
    try {
      await workspace.fs.stat(Uri.file(binPath));
      return binPath;
    } catch {
      // not found, continue to glob search
    }

    // fallback: search with glob
    // maybe use `tinyglobby` later for better performance, VSCode can be slow on globbing large projects.
    const files = await workspace.findFiles(
      new RelativePattern(workspacePath, `**/node_modules/.bin/${binaryName}`),
      null,
      1,
    );

    return files.length > 0 ? files[0].fsPath : undefined;
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
