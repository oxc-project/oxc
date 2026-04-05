import { lstat, rm } from "node:fs/promises";
import {
  REAL_SVELTE_FIXTURE_SPECS,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  TEST_NODE_MODULES_LINK_PATH,
  getFixtureNodeModulesPath,
  getRealSvelteDefaultReportPath,
  getRealSvelteDefaultRunStatePath,
  getRealSvelteInstallRootPath,
} from "./svelte-real-package-metadata.ts";

interface CliOptions {
  keepDiagnostics: boolean;
}

function parseCliOptions(argv: string[]): CliOptions {
  let keepDiagnostics = false;

  for (const arg of argv) {
    if (arg === "--") {
      continue;
    }

    if (arg === "--keep-diagnostics") {
      keepDiagnostics = true;
      continue;
    }

    if (arg === "--help") {
      process.stdout.write(
        "Usage: node --experimental-strip-types ./scripts/cleanup-real-svelte-packages.ts [--keep-diagnostics]\n",
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return { keepDiagnostics };
}

async function removeSymlink(linkPath: string): Promise<boolean> {
  try {
    const stats = await lstat(linkPath);
    if (!stats.isSymbolicLink()) {
      throw new Error(
        `Refusing to remove non-symlink node_modules at ${linkPath}. Remove it manually if it is intentional.`,
      );
    }
    await rm(linkPath, { recursive: true, force: true });
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));
  const removedLabels: string[] = [];

  if (await removeSymlink(TEST_NODE_MODULES_LINK_PATH)) {
    removedLabels.push("test root");
  }

  for (const fixture of REAL_SVELTE_FIXTURE_SPECS) {
    if (await removeSymlink(getFixtureNodeModulesPath(fixture.name))) {
      removedLabels.push(fixture.name);
    }
  }

  const diagnosticPaths = [
    ...new Set([
      ...REAL_SVELTE_PACKAGE_PROFILE_NAMES.flatMap((profileName) => [
        getRealSvelteDefaultReportPath(profileName, "markdown"),
        getRealSvelteDefaultReportPath(profileName, "json"),
        getRealSvelteDefaultRunStatePath(profileName),
      ]),
      ...(process.env.OXLINT_SVELTE_REAL_PACKAGES_STATE_PATH
        ? [process.env.OXLINT_SVELTE_REAL_PACKAGES_STATE_PATH]
        : []),
    ]),
  ];

  const installRootPaths = [...new Set(REAL_SVELTE_PACKAGE_PROFILE_NAMES.map(getRealSvelteInstallRootPath))];

  await Promise.all([
    ...installRootPaths.map((installRootPath) => rm(installRootPath, { recursive: true, force: true })),
    ...(options.keepDiagnostics
      ? []
      : diagnosticPaths.map((diagnosticPath) => rm(diagnosticPath, { force: true }))),
  ]);

  process.stdout.write(
    `Removed real-package Svelte helper artifacts.\n` +
      `Removed node_modules links:\n- ${removedLabels.length === 0 ? "(none)" : removedLabels.join("\n- ")}\n` +
      `Removed install roots:\n- ${installRootPaths.join("\n- ")}\n` +
      `${options.keepDiagnostics ? "Preserved diagnostics:\n" : "Removed diagnostics:\n"}` +
      `- ${diagnosticPaths.join("\n- ")}\n`,
  );
}

await main();
