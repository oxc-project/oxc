import { lstat, readFile, realpath } from "node:fs/promises";
import { createRequire } from "node:module";
import {
  isAbsolute as pathIsAbsolute,
  join as pathJoin,
  relative as pathRelative,
  sep as pathSep,
} from "node:path";
import process from "node:process";
import {
  REAL_SVELTE_FIXTURE_SPECS,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  TEST_NODE_MODULES_LINK_PATH,
  TEST_ROOT_PATH,
  getFixtureNodeModulesPath,
  getRealSvelteDependencySpecifierMap,
  getRealSvelteInstallNodeModulesPath,
  getRealSvelteInstallPackageJsonPath,
  getRealSveltePackageEntries,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";
import { getRealSvelteFixtures } from "../test/real_svelte_fixtures.ts";
import { getMissingPackagesForFixture } from "../test/utils.ts";

import type { Fixture } from "../test/utils.ts";

interface CliOptions {
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number];
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
      process.stdout.write(
        `Usage: node --experimental-strip-types ./scripts/check-real-svelte-packages.ts [--profile <${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join("|")}>]\n`,
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return { profileName };
}

async function assertInstallPackageJson(
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number],
): Promise<void> {
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

  const packageJson = JSON.parse(packageJsonText) as {
    dependencies?: Record<string, string>;
  };

  expectEqualRecords(
    packageJson.dependencies ?? {},
    getRealSvelteDependencySpecifierMap(profileName),
    `Expected ${installPackageJsonPath} to contain the ${profileName} real-package Svelte dependency set.`,
  );
}

async function assertInstalledPackageVersions(
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number],
): Promise<void> {
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);

  for (const { packageName, dependencySpecifier, expectedVersion } of getRealSveltePackageEntries(profileName)) {
    const packageJsonPath = pathJoin(
      installNodeModulesPath,
      ...packageName.split("/"),
      "package.json",
    );
    const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8")) as { version?: string };

    if (typeof packageJson.version !== "string") {
      throw new Error(`Expected ${packageName} to be installed for profile ${profileName}.`);
    }

    if (expectedVersion !== null && packageJson.version !== expectedVersion) {
      throw new Error(
        `Expected ${packageName} version ${expectedVersion}, found ${packageJson.version ?? "unknown"}.`,
      );
    }

    if (expectedVersion === null && packageJson.version.length === 0) {
      throw new Error(
        `Expected ${packageName} installed via ${dependencySpecifier} to report a resolved version.`,
      );
    }
  }
}

async function assertSymlink(
  linkPath: string,
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number],
): Promise<void> {
  const stats = await lstat(linkPath);
  if (!stats.isSymbolicLink()) {
    throw new Error(`Expected ${linkPath} to be a symlink to the shared real-package node_modules directory.`);
  }

  const [resolvedLinkPath, expectedTargetPath] = await Promise.all([
    realpath(linkPath),
    realpath(getRealSvelteInstallNodeModulesPath(profileName)),
  ]);

  if (resolvedLinkPath !== expectedTargetPath) {
    throw new Error(
      `Expected ${linkPath} to resolve to ${expectedTargetPath}, found ${resolvedLinkPath}.`,
    );
  }
}

async function assertPackagesResolveFrom(
  fromDirPath: string,
  packageNames: readonly string[],
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number],
): Promise<void> {
  const expectedNodeModulesPath = await realpath(getRealSvelteInstallNodeModulesPath(profileName));
  const requireFromDir = createRequire(pathJoin(fromDirPath, "__oxlint_real_svelte__.js"));

  for (const packageName of packageNames) {
    const resolvedPath = requireFromDir.resolve(packageName);
    const resolvedRealPath = await realpath(resolvedPath);
    assertPathInside(
      expectedNodeModulesPath,
      resolvedRealPath,
      `Expected ${packageName} resolved from ${fromDirPath} to come from ${expectedNodeModulesPath}, found ${resolvedRealPath}.`,
    );
  }
}

async function assertFixtureResolution(
  fixture: Fixture,
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number],
): Promise<void> {
  const missingPackages = getMissingPackagesForFixture(fixture);
  if (missingPackages.length > 0) {
    throw new Error(
      `Fixture ${fixture.name} is missing required packages after setup: ${missingPackages.join(", ")}.`,
    );
  }

  const fixtureCwd =
    fixture.options.cwd === null ? fixture.dirPath : pathJoin(fixture.dirPath, fixture.options.cwd);

  await assertPackagesResolveFrom(fixture.dirPath, fixture.options.requiredPackages, profileName);
  if (fixtureCwd !== fixture.dirPath) {
    await assertPackagesResolveFrom(fixtureCwd, fixture.options.requiredPackages, profileName);
  }
}

function assertPathInside(parentPath: string, childPath: string, errorMessage: string): void {
  const relativePath = pathRelative(parentPath, childPath);
  const isOutside =
    relativePath === ".." ||
    relativePath.startsWith(`..${pathSep}`) ||
    pathIsAbsolute(relativePath);

  if (isOutside) {
    throw new Error(errorMessage);
  }
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

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));

  await assertInstallPackageJson(options.profileName);
  await assertInstalledPackageVersions(options.profileName);
  await assertSymlink(TEST_NODE_MODULES_LINK_PATH, options.profileName);
  await assertPackagesResolveFrom(
    TEST_ROOT_PATH,
    getRealSveltePackageEntries(options.profileName).map(({ packageName }) => packageName),
    options.profileName,
  );

  for (const fixture of REAL_SVELTE_FIXTURE_SPECS) {
    await assertSymlink(getFixtureNodeModulesPath(fixture.name), options.profileName);
  }

  for (const fixture of getRealSvelteFixtures()) {
    await assertFixtureResolution(fixture, options.profileName);
  }

  process.stdout.write(
    `Verified real-package Svelte test wiring for ${REAL_SVELTE_FIXTURE_SPECS.length} fixtures.\n` +
      `Profile: ${options.profileName}\n`,
  );
}

await main();
