import { rm } from "node:fs/promises";
import process from "node:process";
import {
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH,
  getRealSvelteDefaultReportPath,
  getRealSvelteDefaultRunStatePath,
  getRealSvelteInstallRootPath,
  isRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";

import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";

interface CliOptions {
  keepDiagnostics: boolean;
  profileNames: readonly RealSveltePackageProfileName[];
}

function parseCliOptions(argv: string[]): CliOptions {
  let keepDiagnostics = false;
  let profileNames: readonly RealSveltePackageProfileName[] = REAL_SVELTE_PACKAGE_PROFILE_NAMES;

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
      if (!isRealSveltePackageProfileName(value)) {
        throw new Error(
          `Unknown real-package Svelte profile: ${value}. Expected one of: ${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join(", ")}.`,
        );
      }
      profileNames = [value];
      index += 1;
      continue;
    }

    if (arg === "--all") {
      profileNames = REAL_SVELTE_PACKAGE_PROFILE_NAMES;
      continue;
    }

    if (arg === "--keep-diagnostics") {
      keepDiagnostics = true;
      continue;
    }

    if (arg === "--help") {
      process.stdout.write(
        "Usage: node --experimental-strip-types ./scripts/cleanup-real-svelte-packages.ts [--all] [--profile <pinned|latest-svelte>] [--keep-diagnostics]\n",
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return { keepDiagnostics, profileNames };
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));

  await rm(REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH, { recursive: true, force: true });
  await Promise.all(
    options.profileNames.flatMap((profileName) => {
      const removals = [rm(getRealSvelteInstallRootPath(profileName), { recursive: true, force: true })];
      if (!options.keepDiagnostics) {
        removals.push(
          rm(getRealSvelteDefaultReportPath(profileName, "markdown"), { force: true }),
          rm(getRealSvelteDefaultReportPath(profileName, "json"), { force: true }),
          rm(getRealSvelteDefaultRunStatePath(profileName), { force: true }),
        );
      }
      return removals;
    }),
  );

  process.stdout.write(
    `Cleaned real-package Svelte formatter helper state for profiles: ${options.profileNames.join(", ")}.${options.keepDiagnostics ? " Kept diagnostics." : ""}\n`,
  );
}

await main();
