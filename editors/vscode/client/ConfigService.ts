import * as path from "node:path";
import { promises as fs } from "node:fs";
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
    defaultPattern: string,
  ): Promise<string | undefined> {
    if (!settingsBinary) {
      return this.searchBinaryInWorkspaces(defaultPattern);
    }

    if (!workspace.isTrusted) {
      return;
    }

    // validates the given path is safe to use
    if (validateSafeBinaryPath(settingsBinary) === false) {
      return undefined;
    }

    if (!path.isAbsolute(settingsBinary)) {
      // if the path is not absolute, resolve it to the first workspace folder
      const cwd = this.workspaceConfigs.keys().next().value;
      if (!cwd) {
        return undefined;
      }
      settingsBinary = path.normalize(path.join(cwd, settingsBinary));
      // strip the leading slash on Windows
      if (process.platform === "win32" && settingsBinary.startsWith("\\")) {
        settingsBinary = settingsBinary.slice(1);
      }
    }

    return settingsBinary;
  }

  /**
   * Search for binary in all workspace folders, using optimized strategy:
   * 1. Check direct paths in all workspace node_modules/.bin/ first
   * 2. Fall back to recursive glob search only if needed
   */
  private async searchBinaryInWorkspaces(defaultPattern: string): Promise<string | undefined> {
    const workspacePaths = Array.from(this.workspaceConfigs.keys());

    if (workspacePaths.length === 0) {
      return undefined;
    }

    // First, try direct path checks in all workspace folders
    for (const workspacePath of workspacePaths) {
      const directPath = path.join(workspacePath, "node_modules", ".bin", defaultPattern);
      try {
        await fs.access(directPath);
        return directPath;
      } catch {
        // File doesn't exist, continue to next workspace
      }
    }

    // If direct path checks fail, fall back to recursive glob search
    // Try each workspace folder in order
    for (const workspacePath of workspacePaths) {
      try {
        const files = await workspace.findFiles(
          new RelativePattern(workspacePath, `**/node_modules/.bin/${defaultPattern}`),
          null,
          1,
        );

        if (files.length > 0) {
          return files[0].fsPath;
        }
      } catch {
        // Glob search failed (timeout, permission issues, etc.), try next workspace
      }
    }

    return undefined;
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
