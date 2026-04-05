import { dirname, join } from "node:path";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const DEFAULT_PACKAGE_VERSION = "0.0.0";
const DEFAULT_PACKAGE_NAME = "oxlint-app";

type PackageJson = {
  name?: unknown;
  version?: unknown;
};

function readPackageVersion(packageJsonPath: string, packageName: string): string | null {
  if (!existsSync(packageJsonPath)) return null;

  try {
    const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8")) as PackageJson;
    if (packageJson.name !== packageName) return null;
    return typeof packageJson.version === "string" ? packageJson.version : null;
  } catch {
    return null;
  }
}

export function resolveNearestPackageVersion(
  moduleUrl: string,
  packageName: string = DEFAULT_PACKAGE_NAME,
): string {
  let currentDir = dirname(fileURLToPath(moduleUrl));

  while (true) {
    const version = readPackageVersion(join(currentDir, "package.json"), packageName);
    if (version !== null) return version;

    const parentDir = dirname(currentDir);
    if (parentDir === currentDir) return DEFAULT_PACKAGE_VERSION;
    currentDir = parentDir;
  }
}

export const packageVersion = resolveNearestPackageVersion(import.meta.url);
