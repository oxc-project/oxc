import { spawnSync } from "node:child_process";
import { mkdir, rm, symlink, writeFile } from "node:fs/promises";
import process from "node:process";
import {
  getRealSvelteDependencySpecifierMap,
  getRealSvelteInstallNodeModulesPath,
  getRealSvelteInstallPackageJsonPath,
  getRealSvelteInstallRootPath,
  getRealSveltePackageEntries,
  resolveRealSveltePackageProfileName,
  REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH,
} from "./svelte-real-package-metadata.ts";

import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";

const NPM_COMMAND = process.platform === "win32" ? "npm.cmd" : "npm";

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
      process.stdout.write("Usage: node --experimental-strip-types ./scripts/install-real-svelte-packages.ts [--profile <pinned|latest-svelte>]\n");
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return { profileName };
}

async function writeInstallPackageJson(profileName: RealSveltePackageProfileName): Promise<void> {
  const packageJsonPath = getRealSvelteInstallPackageJsonPath(profileName);
  const dependencies = getRealSvelteDependencySpecifierMap(profileName);

  await mkdir(getRealSvelteInstallRootPath(profileName), { recursive: true });
  await writeFile(
    packageJsonPath,
    `${JSON.stringify(
      {
        name: `oxfmt-real-svelte-packages-${profileName}`,
        private: true,
        type: "module",
        license: "MIT",
        dependencies,
      },
      null,
      2,
    )}\n`,
  );
}

function runNpmInstall(profileName: RealSveltePackageProfileName): void {
  const installRoot = getRealSvelteInstallRootPath(profileName);
  const result = spawnSync(NPM_COMMAND, ["install", "--no-audit", "--no-fund", "--package-lock=false"], {
    cwd: installRoot,
    stdio: "inherit",
  });

  if (result.status !== 0) {
    throw new Error(`npm install failed for profile ${profileName} with exit code ${result.status ?? "unknown"}.`);
  }
}

async function relinkFixtureNodeModules(profileName: RealSveltePackageProfileName): Promise<void> {
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);
  await rm(REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH, { recursive: true, force: true });
  await symlink(installNodeModulesPath, REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH, process.platform === "win32" ? "junction" : "dir");
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));

  await writeInstallPackageJson(options.profileName);
  runNpmInstall(options.profileName);
  await relinkFixtureNodeModules(options.profileName);

  process.stdout.write(
    `Installed ${getRealSveltePackageEntries(options.profileName).length} real-package Svelte formatter dependencies.\n` +
      `Profile: ${options.profileName}\n`,
  );
}

await main();
