import * as fs from "node:fs";
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

  // Cache for binary path lookups to avoid repeated expensive searches
  private binaryPathCache: Map<string, string | undefined> = new Map();

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
    // Invalidate cache when workspace folders change
    this.binaryPathCache.clear();
  }

  public removeWorkspaceConfig(workspace: WorkspaceFolder): void {
    this.workspaceConfigs.delete(workspace.uri.path);
    // Invalidate cache when workspace folders change
    this.binaryPathCache.clear();
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

  /**
   * Checks if a binary exists at the given path.
   * Handles both Unix and Windows path differences.
   */
  private checkBinaryExists(binaryPath: string): boolean {
    try {
      // Check if file exists and is executable (on Unix) or accessible (on Windows)
      fs.accessSync(binaryPath, fs.constants.F_OK);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Searches for a binary in node_modules/.bin directories using file system operations.
   * This is much faster than workspace.findFiles in large monorepos.
   * Limits search depth and scope to avoid timeouts.
   */
  private async searchBinaryInNodeModules(
    startPath: string,
    binaryName: string,
    maxDepth: number = 3,
    currentDepth: number = 0,
  ): Promise<string | undefined> {
    // Limit depth to avoid deep recursion in large monorepos
    if (currentDepth >= maxDepth) {
      return undefined;
    }

    try {
      const nodeModulesPath = path.join(startPath, "node_modules", ".bin");

      // Check if this .bin directory exists
      if (fs.existsSync(nodeModulesPath)) {
        const binaryPath = path.join(nodeModulesPath, binaryName);
        if (this.checkBinaryExists(binaryPath)) {
          return binaryPath;
        }

        // Check with .exe extension on Windows
        if (process.platform === "win32") {
          const exePath = `${binaryPath}.exe`;
          if (this.checkBinaryExists(exePath)) {
            return exePath;
          }
        }
      }

      // Search in parent directories up to workspace root
      const parentPath = path.dirname(startPath);

      // Stop if we've reached the filesystem root or if parent is same as current
      if (parentPath === startPath || parentPath === "/" || parentPath.match(/^[A-Z]:\\?$/)) {
        return undefined;
      }

      // Recursively search parent directory
      return this.searchBinaryInNodeModules(parentPath, binaryName, maxDepth, currentDepth + 1);
    } catch {
      // Ignore errors (permission denied, etc.) and continue searching
      return undefined;
    }
  }

  /**
   * Wraps workspace.findFiles with a timeout to prevent indefinite hanging.
   * Returns undefined if the search times out or fails.
   */
  private async findFilesWithTimeout(
    pattern: RelativePattern,
    exclude: string | null,
    maxResults: number,
    timeoutMs: number = 5000,
  ): Promise<Uri[]> {
    try {
      const searchPromise = workspace.findFiles(pattern, exclude, maxResults);
      const timeoutPromise = new Promise<Uri[]>((_, reject) => {
        setTimeout(() => reject(new Error("Binary search timeout")), timeoutMs);
      });

      return await Promise.race([searchPromise, timeoutPromise]);
    } catch {
      // Timeout or other error - return empty array to indicate failure
      // This allows graceful degradation without throwing
      return [];
    }
  }

  private async searchBinaryPath(
    settingsBinary: string | undefined,
    defaultPattern: string,
  ): Promise<string | undefined> {
    const cwd = this.workspaceConfigs.keys().next().value;
    if (!cwd) {
      return undefined;
    }

    if (!settingsBinary) {
      // Check cache first
      const cacheKey = `auto:${defaultPattern}`;
      const cached = this.binaryPathCache.get(cacheKey);
      if (cached !== undefined) {
        return cached;
      }

      // Fast path: Check common locations first using file system operations
      // This is much faster than recursive findFiles in large monorepos
      const workspaceFolders = Array.from(this.workspaceConfigs.keys());

      // Step 1: Check root node_modules/.bin in each workspace folder
      for (const workspacePath of workspaceFolders) {
        const rootBinPath = path.join(workspacePath, "node_modules", ".bin", defaultPattern);
        if (this.checkBinaryExists(rootBinPath)) {
          this.binaryPathCache.set(cacheKey, rootBinPath);
          return rootBinPath;
        }

        // Also check with .exe extension on Windows
        if (process.platform === "win32") {
          const exePath = `${rootBinPath}.exe`;
          if (this.checkBinaryExists(exePath)) {
            this.binaryPathCache.set(cacheKey, exePath);
            return exePath;
          }
        }
      }

      // Step 2: Search parent directories up to workspace root using file system operations
      // This handles cases where binaries are in nested packages
      // Search all workspace folders in parallel for better performance
      const searchPromises = workspaceFolders.map((workspacePath) =>
        this.searchBinaryInNodeModules(workspacePath, defaultPattern, 3),
      );
      const searchResults = await Promise.all(searchPromises);
      const found = searchResults.find((result) => result !== undefined);
      if (found) {
        this.binaryPathCache.set(cacheKey, found);
        return found;
      }

      // Step 3: Last resort - use findFiles with timeout protection
      // Only search in workspace folders, not deep recursion
      // Use Promise.race to add timeout protection
      try {
        const excludePattern = "**/{.pnpm,node_modules/.pnpm}/**";
        const files = await this.findFilesWithTimeout(
          new RelativePattern(cwd, `**/node_modules/.bin/${defaultPattern}`),
          excludePattern,
          1,
          5000, // 5 second timeout
        );

        const result = files.length > 0 ? files[0].fsPath : undefined;
        this.binaryPathCache.set(cacheKey, result);
        return result;
      } catch {
        // Timeout or other error - cache undefined to avoid retrying
        this.binaryPathCache.set(cacheKey, undefined);
        return undefined;
      }
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
      settingsBinary = path.normalize(path.join(cwd, settingsBinary));
      // strip the leading slash on Windows
      if (process.platform === "win32" && settingsBinary.startsWith("\\")) {
        settingsBinary = settingsBinary.slice(1);
      }
    }

    return settingsBinary;
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
