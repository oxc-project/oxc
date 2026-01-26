import { spawnSync } from "node:child_process";
import { homedir } from "node:os";
import * as path from "node:path";
import { Uri, workspace } from "vscode";
import { validateSafeBinaryPath } from "./PathValidator";

function replaceTargetFromMainToBin(resolvedPath: string, binaryName: string): string {
  // we want to target the binary instead of the main index file
  // Improvement: search inside package.json "bin" and `main` field for more reliability
  return resolvedPath.replace(
    `${binaryName}${path.sep}dist${path.sep}index.js`,
    `${binaryName}${path.sep}bin${path.sep}${binaryName}`,
  );
}
/**
 * Search for the binary in all workspaces' node_modules/.bin directories.
 * If multiple workspaces contain the binary, the first one found is returned.
 */
export async function searchProjectNodeModulesBin(binaryName: string): Promise<string | undefined> {
  // try to resolve via require.resolve
  try {
    const resolvedPath = replaceTargetFromMainToBin(
      require.resolve(binaryName, {
        paths: workspace.workspaceFolders?.map((folder) => folder.uri.fsPath) ?? [],
      }),
      binaryName,
    );
    return resolvedPath;
  } catch {}
}

/**
 * Search for the binary in global node_modules.
 * Returns undefined if not found.
 */
export async function searchGlobalNodeModulesBin(binaryName: string): Promise<string | undefined> {
  // try to resolve via require.resolve
  try {
    const resolvedPath = replaceTargetFromMainToBin(
      require.resolve(binaryName, { paths: globalNodeModulesPaths() }),
      binaryName,
    );
    return resolvedPath;
  } catch {}
}

/**
 * Search for the binary based on user settings.
 * If the path is relative, it is resolved against the first workspace folder.
 * Returns undefined if no valid binary is found or the path is unsafe.
 */
export async function searchSettingsBin(settingsBinary: string): Promise<string | undefined> {
  if (!workspace.isTrusted) {
    return;
  }

  // validates the given path is safe to use
  if (!validateSafeBinaryPath(settingsBinary)) {
    return undefined;
  }

  if (!path.isAbsolute(settingsBinary)) {
    const cwd = workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!cwd) {
      return undefined;
    }
    // if the path is not absolute, resolve it to the first workspace folder
    settingsBinary = path.normalize(path.join(cwd, settingsBinary));
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

// copied from: https://github.com/biomejs/biome-vscode/blob/ae9b6df2254d0ff8ee9d626554251600eb2ca118/src/locator.ts#L28-L49
function globalNodeModulesPaths(): string[] {
  const npmGlobalNodeModulesPath = safeSpawnSync("npm", ["root", "-g"]);
  const pnpmGlobalNodeModulesPath = safeSpawnSync("pnpm", ["root", "-g"]);
  const bunGlobalNodeModulesPath = path.resolve(homedir(), ".bun/install/global/node_modules");

  return [npmGlobalNodeModulesPath, pnpmGlobalNodeModulesPath, bunGlobalNodeModulesPath].filter(
    Boolean,
  ) as string[];
}

// only use this function with internal code, because it executes shell commands
// which could be a security risk if the command or args are user-controlled
const safeSpawnSync = (command: string, args: readonly string[] = []): string | undefined => {
  let output: string | undefined;

  try {
    const result = spawnSync(command, args, {
      shell: true,
      encoding: "utf8",
    });

    if (result.error || result.status !== 0) {
      output = undefined;
    } else {
      const trimmed = result.stdout.trim();
      output = trimmed ? trimmed : undefined;
    }
  } catch {
    output = undefined;
  }

  return output;
};
