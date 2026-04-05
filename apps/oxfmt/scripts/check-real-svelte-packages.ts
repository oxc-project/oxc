import { spawnSync } from "node:child_process";
import { lstat, readFile } from "node:fs/promises";
import { join as pathJoin } from "node:path";
import process from "node:process";
import {
  REAL_SVELTE_REQUIRED_PACKAGES,
  REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH,
  REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
  REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH,
  getRealSvelteDependencySpecifierMap,
  getRealSvelteExpectedVersionMap,
  getRealSvelteInstallNodeModulesPath,
  getRealSvelteInstallPackageJsonPath,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";

import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";

interface CliOptions {
  profileName: RealSveltePackageProfileName;
}

function parseCliOptions(argv: string[]): CliOptions {
  let profileName = resolveRealSveltePackageProfileName(undefined);

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--") {
      continue;
    }
    if (arg === "--profile") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("Expected a profile name after --profile.");
      }
      profileName = resolveRealSveltePackageProfileName(value);
      index += 1;
      continue;
    }

    if (arg === "--help") {
      process.stdout.write("Usage: node --experimental-strip-types ./scripts/check-real-svelte-packages.ts [--profile <pinned|latest-svelte>]\n");
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return { profileName };
}

function expectEqualRecords(
  actual: Record<string, string>,
  expected: Record<string, string>,
  errorMessage: string,
): void {
  const actualEntries = Object.entries(actual).sort(([left], [right]) => left.localeCompare(right));
  const expectedEntries = Object.entries(expected).sort(([left], [right]) => left.localeCompare(right));

  if (JSON.stringify(actualEntries) !== JSON.stringify(expectedEntries)) {
    throw new Error(errorMessage);
  }
}

function getInstalledPackageJsonPath(installNodeModulesPath: string, packageName: string): string {
  return pathJoin(installNodeModulesPath, ...packageName.split("/"), "package.json");
}

function canImportPackage(packageName: string, fromDirPath: string): boolean {
  const result = spawnSync(
    process.execPath,
    ["--input-type=module", "--eval", `await import(${JSON.stringify(packageName)});`],
    {
      cwd: fromDirPath,
      stdio: "ignore",
    },
  );

  return result.status === 0;
}

async function assertInstallPackageJson(profileName: RealSveltePackageProfileName): Promise<void> {
  const installPackageJsonPath = getRealSvelteInstallPackageJsonPath(profileName);
  let packageJsonText: string;

  try {
    packageJsonText = await readFile(installPackageJsonPath, "utf8");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      throw new Error(
        `Missing ${installPackageJsonPath}. Run pnpm run setup:svelte-real-packages${
          profileName === "pinned" ? "" : ":latest-svelte"
        } first.`,
      );
    }
    throw error;
  }

  const packageJson = JSON.parse(packageJsonText) as { dependencies?: Record<string, string> };

  expectEqualRecords(
    packageJson.dependencies ?? {},
    { ...getRealSvelteDependencySpecifierMap(profileName) },
    `Helper package.json does not match the expected dependency set for profile ${profileName}.`,
  );
}

async function assertInstalledPackageVersions(profileName: RealSveltePackageProfileName): Promise<void> {
  const expectedVersions = getRealSvelteExpectedVersionMap(profileName);
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);

  for (const [packageName, expectedVersion] of Object.entries(expectedVersions)) {
    const packageJsonText = await readFile(getInstalledPackageJsonPath(installNodeModulesPath, packageName), "utf8");
    const packageJson = JSON.parse(packageJsonText) as { version?: string };
    if (packageJson.version !== expectedVersion) {
      throw new Error(
        `Expected ${packageName}@${expectedVersion} in the helper install, got ${packageJson.version ?? "unknown"}.`,
      );
    }
  }
}

async function assertFixtureNodeModulesLinkExists(): Promise<void> {
  const stats = await lstat(REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH);
  if (!stats.isSymbolicLink() && process.platform !== "win32") {
    throw new Error("Expected the runtime real-package fixture node_modules path to be a symlink.");
  }
}

function assertPackagesResolveFrom(fromDirPath: string, packageNames: readonly string[]): void {
  const missingPackages = packageNames.filter((packageName) => !canImportPackage(packageName, fromDirPath));
  if (missingPackages.length > 0) {
    throw new Error(
      `Missing real-package Svelte formatter dependencies from ${fromDirPath}: ${missingPackages.join(", ")}.`,
    );
  }
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));

  await assertInstallPackageJson(options.profileName);
  await assertInstalledPackageVersions(options.profileName);
  await assertFixtureNodeModulesLinkExists();
  assertPackagesResolveFrom(REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH, REAL_SVELTE_REQUIRED_PACKAGES);
  assertPackagesResolveFrom(REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH, REAL_SVELTE_REQUIRED_PACKAGES);

  process.stdout.write(
    "Verified real-package Svelte formatter test wiring.\n" +
      `Profile: ${options.profileName}\n` +
      `Packages: ${REAL_SVELTE_REQUIRED_PACKAGES.join(", ")}\n`,
  );
}

await main();
