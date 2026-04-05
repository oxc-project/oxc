import { spawnSync } from "node:child_process";
import { access, lstat, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { dirname as pathDirname, join as pathJoin, relative as pathRelative } from "node:path";
import process from "node:process";
import {
  PACKAGE_ROOT_PATH,
  REAL_SVELTE_FIXTURE_SPECS,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  getRealSvelteDependencySpecifierMap,
  getRealSveltePackageEntries,
  getRealSvelteInstallNodeModulesPath,
  getRealSvelteInstallPackageJsonPath,
  getRealSvelteInstallRootPath,
  profileHasFloatingRealSveltePackages,
  resolveRealSveltePackageProfileName,
  TEST_NODE_MODULES_LINK_PATH,
  getFixtureDirPath,
  getFixtureNodeModulesPath,
} from "./svelte-real-package-metadata.ts";

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
        `Usage: node --experimental-strip-types ./scripts/install-real-svelte-packages.ts [--profile <${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join("|")}>]\n`,
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return { profileName };
}

async function assertFixtureDirectoriesExist(): Promise<void> {
  for (const fixture of REAL_SVELTE_FIXTURE_SPECS) {
    const dirPath = getFixtureDirPath(fixture.name);
    try {
      await access(dirPath);
    } catch {
      throw new Error(`Missing real-package Svelte fixture directory: ${dirPath}`);
    }
  }
}

async function ensureInstallRoot(profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number]): Promise<void> {
  const installRootPath = getRealSvelteInstallRootPath(profileName);
  const installPackageJsonPath = getRealSvelteInstallPackageJsonPath(profileName);

  await mkdir(installRootPath, { recursive: true });
  await writeFile(
    installPackageJsonPath,
    JSON.stringify(
      {
        name: "oxlint-real-svelte-fixture-packages",
        private: true,
        dependencies: getRealSvelteDependencySpecifierMap(profileName),
      },
      null,
      2,
    ) + "\n",
  );
}

async function hasExpectedInstall(profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number]): Promise<boolean> {
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);

  try {
    for (const { packageName, expectedVersion } of getRealSveltePackageEntries(profileName)) {
      const packageJsonPath = pathJoin(installNodeModulesPath, ...packageName.split("/"), "package.json");
      const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8")) as {
        version?: string;
      };

      if (expectedVersion !== null && packageJson.version !== expectedVersion) {
        return false;
      }

      if (expectedVersion === null && typeof packageJson.version !== "string") {
        return false;
      }
    }

    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return false;
    }

    throw error;
  }
}

function installPackages(profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number]): void {
  const installRootPath = getRealSvelteInstallRootPath(profileName);
  const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
  const result = spawnSync(
    npmCommand,
    [
      "install",
      "--no-package-lock",
      "--no-audit",
      "--no-fund",
      "--prefix",
      installRootPath,
    ],
    {
      cwd: PACKAGE_ROOT_PATH,
      stdio: "inherit",
      env: {
        ...process.env,
        OXLINT_SVELTE_REAL_PACKAGES_PROFILE: profileName,
      },
    },
  );

  if (result.status !== 0) {
    throw new Error(`Failed to install real Svelte packages (exit code ${result.status ?? "unknown"}).`);
  }
}

async function recreateSymlink(
  linkPath: string,
  profileName: (typeof REAL_SVELTE_PACKAGE_PROFILE_NAMES)[number],
): Promise<void> {
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);

  try {
    const stats = await lstat(linkPath);
    if (!stats.isSymbolicLink()) {
      throw new Error(
        `Refusing to replace non-symlink node_modules at ${linkPath}. Remove it manually and rerun setup.`,
      );
    }
    await rm(linkPath, { recursive: true, force: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }

  const symlinkTarget =
    process.platform === "win32"
      ? installNodeModulesPath
      : pathRelative(pathDirname(linkPath), installNodeModulesPath);

  await symlink(symlinkTarget, linkPath, process.platform === "win32" ? "junction" : "dir");
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));

  await assertFixtureDirectoriesExist();
  await ensureInstallRoot(options.profileName);

  const reusedExistingInstall =
    !profileHasFloatingRealSveltePackages(options.profileName) &&
    (await hasExpectedInstall(options.profileName));

  if (!reusedExistingInstall) {
    installPackages(options.profileName);
  }

  const linkedPaths = [{ label: "test root", linkPath: TEST_NODE_MODULES_LINK_PATH }];
  for (const fixture of REAL_SVELTE_FIXTURE_SPECS) {
    linkedPaths.push({
      label: fixture.name,
      linkPath: getFixtureNodeModulesPath(fixture.name),
    });
  }

  for (const linkedPath of linkedPaths) {
    await recreateSymlink(linkedPath.linkPath, options.profileName);
  }

  process.stdout.write(
    `${reusedExistingInstall ? "Reused" : "Installed"} real Svelte fixture packages in ${getRealSvelteInstallRootPath(options.profileName)}\n` +
      `Profile: ${options.profileName}\n` +
      `Pinned/floating package policy: ${getRealSveltePackageEntries(options.profileName)
        .map(({ packageName, dependencySpecifier, expectedVersion }) =>
          expectedVersion === null
            ? `${packageName}@${dependencySpecifier} (floating)`
            : `${packageName}@${expectedVersion}`,
        )
        .join(", ")}\n` +
      `Linked node_modules paths:\n- ${linkedPaths.map(({ label }) => label).join("\n- ")}\n`,
  );
}

await main();
