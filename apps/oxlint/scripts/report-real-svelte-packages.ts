import { spawnSync } from "node:child_process";
import { lstat, mkdir, readFile, realpath, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import {
  dirname as pathDirname,
  isAbsolute as pathIsAbsolute,
  join as pathJoin,
  relative as pathRelative,
  sep as pathSep,
} from "node:path";
import process from "node:process";
import {
  PACKAGE_ROOT_PATH,
  REAL_SVELTE_FIXTURE_SPECS,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  TEST_NODE_MODULES_LINK_PATH,
  TEST_ROOT_PATH,
  getFixtureNodeModulesPath,
  getRealSvelteDefaultReportPath,
  getRealSvelteDefaultRunStatePath,
  getRealSvelteDependencySpecifierMap,
  getRealSvelteInstallNodeModulesPath,
  getRealSvelteInstallPackageJsonPath,
  getRealSvelteInstallRootPath,
  getRealSveltePackageEntries,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";
import { getRealSvelteFixtures } from "../test/real_svelte_fixtures.ts";
import { getMissingPackagesForFixture } from "../test/utils.ts";
import {
  getFailedRealSvelteManagedRunSteps,
  readRealSvelteManagedRunState,
} from "./svelte-real-package-run-state.ts";

import type { Fixture } from "../test/utils.ts";
import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";
import type { RealSvelteManagedRunState } from "./svelte-real-package-run-state.ts";

interface CliOptions {
  format: "json" | "markdown";
  outputPath: string | null;
  profileName: RealSveltePackageProfileName;
  statePath: string;
}

interface PackageReport {
  packageName: string;
  requestedSpecifier: string;
  expectedVersion: string | null;
  installedVersion: string | null;
  status: "ok" | "missing" | "version-mismatch";
  packageJsonPath: string;
}

interface LinkReport {
  label: string;
  linkPath: string;
  exists: boolean;
  isSymlink: boolean;
  resolvedPath: string | null;
  expectedTargetPath: string | null;
  status: "ok" | "missing" | "not-symlink" | "wrong-target";
  error: string | null;
}

interface ResolutionReport {
  scopeLabel: string;
  fromDirPath: string;
  packageName: string;
  resolvedPath: string | null;
  realPath: string | null;
  insideInstallNodeModules: boolean | null;
  error: string | null;
}

interface FixtureReport {
  name: string;
  dirPath: string;
  cwdPath: string;
  requiredPackages: readonly string[];
  missingPackages: readonly string[];
  status: "ok" | "missing-packages";
  link: LinkReport;
  resolutions: ResolutionReport[];
}

interface NpmLsReport {
  exitCode: number | null;
  stdout: string;
  stderr: string;
}

interface RealSvelteReport {
  generatedAt: string;
  profileName: RealSveltePackageProfileName;
  platform: NodeJS.Platform;
  nodeVersion: string;
  packageRootPath: string;
  installRootPath: string;
  installNodeModulesPath: string;
  installPackageJsonPath: string;
  installPackageJsonMatches: boolean;
  managedRunState: RealSvelteManagedRunState | null;
  managedRunStatePath: string;
  installPackageJsonDependencies: Record<string, string>;
  requestedDependencySpecifiers: Record<string, string>;
  packages: PackageReport[];
  links: LinkReport[];
  testRootResolutions: ResolutionReport[];
  fixtures: FixtureReport[];
  npmLs: NpmLsReport;
}

function parseCliOptions(argv: string[]): CliOptions {
  let format: CliOptions["format"] = "markdown";
  let outputPath: string | null = null;
  let statePath: string | null = process.env.OXLINT_SVELTE_REAL_PACKAGES_STATE_PATH ?? null;
  let useDefaultOutput = false;
  let profileName = resolveRealSveltePackageProfileName(undefined);

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === "--") {
      continue;
    }

    if (arg === "--format") {
      const value = argv[index + 1];
      if (value !== "json" && value !== "markdown") {
        throw new Error(`Expected --format to be one of: json, markdown. Received: ${value ?? "<missing>"}.`);
      }
      format = value;
      index += 1;
      continue;
    }

    if (arg === "--output") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("Expected a file path after --output.");
      }
      outputPath = value;
      index += 1;
      continue;
    }

    if (arg === "--state-path") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("Expected a file path after --state-path.");
      }
      statePath = value;
      index += 1;
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

    if (arg === "--default-output") {
      useDefaultOutput = true;
      continue;
    }

    if (arg === "--help") {
      process.stdout.write(
        `Usage: node --experimental-strip-types ./scripts/report-real-svelte-packages.ts [--format markdown|json] [--profile <${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join("|")}>] [--state-path <path>] [--output <path>] [--default-output]\n`,
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  if (useDefaultOutput) {
    outputPath = getRealSvelteDefaultReportPath(profileName, format);
  }

  if (outputPath === null && process.env.OXLINT_SVELTE_REAL_PACKAGES_REPORT_PATH) {
    outputPath = process.env.OXLINT_SVELTE_REAL_PACKAGES_REPORT_PATH;
  }

  return {
    format,
    outputPath,
    profileName,
    statePath: statePath ?? getRealSvelteDefaultRunStatePath(profileName),
  };
}

async function safeReadJson(path: string): Promise<Record<string, unknown> | null> {
  try {
    return JSON.parse(await readFile(path, "utf8")) as Record<string, unknown>;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return null;
    }

    throw error;
  }
}

async function safeRealpath(path: string): Promise<string | null> {
  try {
    return await realpath(path);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return null;
    }

    throw error;
  }
}

function isPathInside(parentPath: string, childPath: string): boolean {
  const relativePath = pathRelative(parentPath, childPath);
  return !(
    relativePath === ".." ||
    relativePath.startsWith(`..${pathSep}`) ||
    pathIsAbsolute(relativePath)
  );
}

async function inspectInstallPackageJson(profileName: RealSveltePackageProfileName): Promise<{
  installPackageJsonMatches: boolean;
  dependencies: Record<string, string>;
}> {
  const installPackageJsonPath = getRealSvelteInstallPackageJsonPath(profileName);
  const packageJson = (await safeReadJson(installPackageJsonPath)) ?? {};
  const dependencies = packageJson.dependencies;
  const normalizedDependencies: Record<string, string> =
    typeof dependencies === "object" && dependencies !== null
      ? Object.fromEntries(
          Object.entries(dependencies).filter(
            (entry): entry is [string, string] => typeof entry[0] === "string" && typeof entry[1] === "string",
          ),
        )
      : {};
  const requestedDependencySpecifiers = getRealSvelteDependencySpecifierMap(profileName);

  return {
    installPackageJsonMatches:
      JSON.stringify(Object.entries(normalizedDependencies).sort()) ===
      JSON.stringify(Object.entries(requestedDependencySpecifiers).sort()),
    dependencies: normalizedDependencies,
  };
}

async function inspectInstalledPackages(
  profileName: RealSveltePackageProfileName,
): Promise<PackageReport[]> {
  const reports: PackageReport[] = [];
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);

  for (const { packageName, dependencySpecifier, expectedVersion } of getRealSveltePackageEntries(profileName)) {
    const packageJsonPath = pathJoin(installNodeModulesPath, ...packageName.split("/"), "package.json");
    const packageJson = await safeReadJson(packageJsonPath);
    const installedVersion = typeof packageJson?.version === "string" ? packageJson.version : null;

    reports.push({
      packageName,
      requestedSpecifier: dependencySpecifier,
      expectedVersion,
      installedVersion,
      status:
        installedVersion === null
          ? "missing"
          : expectedVersion === null || installedVersion === expectedVersion
            ? "ok"
            : "version-mismatch",
      packageJsonPath,
    });
  }

  return reports;
}

async function inspectLink(
  label: string,
  linkPath: string,
  profileName: RealSveltePackageProfileName,
): Promise<LinkReport> {
  const expectedTargetPath = await safeRealpath(getRealSvelteInstallNodeModulesPath(profileName));

  try {
    const stats = await lstat(linkPath);
    const isSymlink = stats.isSymbolicLink();
    const resolvedPath = await safeRealpath(linkPath);

    if (!isSymlink) {
      return {
        label,
        linkPath,
        exists: true,
        isSymlink: false,
        resolvedPath,
        expectedTargetPath,
        status: "not-symlink",
        error: null,
      };
    }

    return {
      label,
      linkPath,
      exists: true,
      isSymlink: true,
      resolvedPath,
      expectedTargetPath,
      status:
        resolvedPath !== null && expectedTargetPath !== null && resolvedPath === expectedTargetPath
          ? "ok"
          : "wrong-target",
      error: null,
    };
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return {
        label,
        linkPath,
        exists: false,
        isSymlink: false,
        resolvedPath: null,
        expectedTargetPath,
        status: "missing",
        error: null,
      };
    }

    return {
      label,
      linkPath,
      exists: false,
      isSymlink: false,
      resolvedPath: null,
      expectedTargetPath,
      status: "missing",
      error: error instanceof Error ? error.stack ?? error.message : String(error),
    };
  }
}

async function inspectResolution(
  scopeLabel: string,
  fromDirPath: string,
  packageName: string,
  profileName: RealSveltePackageProfileName,
): Promise<ResolutionReport> {
  const expectedNodeModulesPath = await safeRealpath(getRealSvelteInstallNodeModulesPath(profileName));

  try {
    const requireFromDir = createRequire(pathJoin(fromDirPath, "__oxlint_real_svelte_report__.js"));
    const resolvedPath = requireFromDir.resolve(packageName);
    const realPath = await safeRealpath(resolvedPath);

    return {
      scopeLabel,
      fromDirPath,
      packageName,
      resolvedPath,
      realPath,
      insideInstallNodeModules:
        expectedNodeModulesPath !== null && realPath !== null
          ? isPathInside(expectedNodeModulesPath, realPath)
          : null,
      error: null,
    };
  } catch (error) {
    return {
      scopeLabel,
      fromDirPath,
      packageName,
      resolvedPath: null,
      realPath: null,
      insideInstallNodeModules: null,
      error: error instanceof Error ? error.stack ?? error.message : String(error),
    };
  }
}

async function inspectFixture(
  fixture: Fixture,
  profileName: RealSveltePackageProfileName,
): Promise<FixtureReport> {
  const cwdPath =
    fixture.options.cwd === null ? fixture.dirPath : pathJoin(fixture.dirPath, fixture.options.cwd);
  const resolutions: ResolutionReport[] = [];

  for (const packageName of fixture.options.requiredPackages) {
    resolutions.push(await inspectResolution(`${fixture.name}:fixture`, fixture.dirPath, packageName, profileName));
    if (cwdPath !== fixture.dirPath) {
      resolutions.push(await inspectResolution(`${fixture.name}:cwd`, cwdPath, packageName, profileName));
    }
  }

  const missingPackages = getMissingPackagesForFixture(fixture);

  return {
    name: fixture.name,
    dirPath: fixture.dirPath,
    cwdPath,
    requiredPackages: fixture.options.requiredPackages,
    missingPackages,
    status: missingPackages.length === 0 ? "ok" : "missing-packages",
    link: await inspectLink(fixture.name, getFixtureNodeModulesPath(fixture.name), profileName),
    resolutions,
  };
}

function inspectNpmLs(profileName: RealSveltePackageProfileName): NpmLsReport {
  const installRootPath = getRealSvelteInstallRootPath(profileName);
  const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
  const result = spawnSync(
    npmCommand,
    ["ls", "--depth=0", "--json", "--prefix", installRootPath],
    {
      cwd: PACKAGE_ROOT_PATH,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  return {
    exitCode: result.status,
    stdout: result.stdout?.toString() ?? "",
    stderr: result.stderr?.toString() ?? "",
  };
}

async function buildReport(
  profileName: RealSveltePackageProfileName,
  managedRunStatePath: string,
): Promise<RealSvelteReport> {
  const installPackageJson = await inspectInstallPackageJson(profileName);
  const fixtures = getRealSvelteFixtures();
  const testRootPackages = [...new Set(getRealSveltePackageEntries(profileName).map(({ packageName }) => packageName))];
  const testRootResolutions: ResolutionReport[] = [];

  for (const packageName of testRootPackages) {
    testRootResolutions.push(await inspectResolution("test-root", TEST_ROOT_PATH, packageName, profileName));
  }

  const installRootPath = getRealSvelteInstallRootPath(profileName);
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);
  const installPackageJsonPath = getRealSvelteInstallPackageJsonPath(profileName);

  return {
    generatedAt: new Date().toISOString(),
    profileName,
    platform: process.platform,
    nodeVersion: process.version,
    packageRootPath: PACKAGE_ROOT_PATH,
    installRootPath,
    installNodeModulesPath,
    installPackageJsonPath,
    installPackageJsonMatches: installPackageJson.installPackageJsonMatches,
    managedRunState: await readRealSvelteManagedRunState(managedRunStatePath),
    managedRunStatePath,
    installPackageJsonDependencies: installPackageJson.dependencies,
    requestedDependencySpecifiers: getRealSvelteDependencySpecifierMap(profileName),
    packages: await inspectInstalledPackages(profileName),
    links: [
      await inspectLink("test root", TEST_NODE_MODULES_LINK_PATH, profileName),
      ...(await Promise.all(
        REAL_SVELTE_FIXTURE_SPECS.map((fixture) =>
          inspectLink(fixture.name, getFixtureNodeModulesPath(fixture.name), profileName),
        ),
      )),
    ],
    testRootResolutions,
    fixtures: await Promise.all(fixtures.map((fixture) => inspectFixture(fixture, profileName))),
    npmLs: inspectNpmLs(profileName),
  };
}

function renderStatus(status: string): string {
  if (status === "ok" || status === "passed") {
    return "✅";
  }
  if (status === "running") {
    return "⏳";
  }
  return "❌";
}

function renderResolutionLine(resolution: ResolutionReport): string {
  if (resolution.error !== null) {
    return `- ❌ ${resolution.scopeLabel} → ${resolution.packageName}: ${resolution.error}`;
  }

  return (
    `- ${resolution.insideInstallNodeModules ? "✅" : "⚠️"} ${resolution.scopeLabel} → ${resolution.packageName}: ` +
    `${resolution.realPath ?? resolution.resolvedPath ?? "<unresolved>"}`
  );
}

function renderInstalledPackageLine(item: PackageReport): string {
  if (item.expectedVersion === null) {
    return (
      `- ${renderStatus(item.status)} ${item.packageName}: requested ${item.requestedSpecifier}, ` +
      `resolved ${item.installedVersion ?? "<missing>"}`
    );
  }

  return (
    `- ${renderStatus(item.status)} ${item.packageName}: requested ${item.requestedSpecifier}, ` +
    `expected ${item.expectedVersion}, found ${item.installedVersion ?? "<missing>"}`
  );
}

function renderManagedRunStepLine(step: RealSvelteManagedRunState["steps"][number]): string {
  const commandSuffix =
    step.extraArgs.length === 0 ? step.scriptName : `${step.scriptName} -- ${step.extraArgs.join(" ")}`;
  const exitCodeSuffix = step.exitCode === null ? "" : ` (exit ${step.exitCode})`;
  const errorSuffix = step.errorMessage === null ? "" : ` — ${step.errorMessage}`;
  return `- ${renderStatus(step.status)} ${step.stepName}: ${commandSuffix}${exitCodeSuffix}${errorSuffix}`;
}

function renderMarkdown(report: RealSvelteReport): string {
  const failingPackages = report.packages.filter((item) => item.status !== "ok");
  const failingLinks = report.links.filter((item) => item.status !== "ok");
  const failingFixtures = report.fixtures.filter((item) => item.status !== "ok");
  const failingResolutions = [
    ...report.testRootResolutions,
    ...report.fixtures.flatMap((fixture) => fixture.resolutions),
  ].filter((resolution) => resolution.error !== null || resolution.insideInstallNodeModules === false);
  const failingManagedRunSteps = getFailedRealSvelteManagedRunSteps(report.managedRunState);

  const lines = [
    "# Oxlint real-package Svelte diagnostic report",
    "",
    `- Generated: ${report.generatedAt}`,
    `- Profile: ${report.profileName}`,
    `- Platform: ${report.platform}`,
    `- Node: ${report.nodeVersion}`,
    `- Package root: ${report.packageRootPath}`,
    `- Install root: ${report.installRootPath}`,
    `- Expected package count: ${report.packages.length}`,
    `- Fixture count: ${report.fixtures.length}`,
    `- Managed run state path: ${report.managedRunStatePath}`,
    "",
    "## Summary",
    "",
    `- Install package.json matches ${report.profileName} manifest: ${report.installPackageJsonMatches ? "yes" : "no"}`,
    `- Package failures: ${failingPackages.length}`,
    `- Link failures: ${failingLinks.length}`,
    `- Fixture failures: ${failingFixtures.length}`,
    `- Resolution failures: ${failingResolutions.length}`,
    `- Managed run failures: ${failingManagedRunSteps.length}`,
    "",
    ...(report.managedRunState === null
      ? [
          "## Managed run",
          "",
          "- Status: ℹ️ no managed run state found",
          "",
        ]
      : [
          "## Managed run",
          "",
          `- Status: ${renderStatus(report.managedRunState.laneStatus)} ${report.managedRunState.laneStatus}`,
          `- Requested suites: ${report.managedRunState.requestedSuites.join(", ") || "(none)"}`,
          `- Build requested: ${report.managedRunState.build ? "yes" : "no"}`,
          `- Keep on failure: ${report.managedRunState.keepOnFailure ? "yes" : "no"}`,
          `- Report mode: ${report.managedRunState.reportMode}`,
          `- Failure message: ${report.managedRunState.failureMessage ?? "(none)"}`,
          "",
          "### Recorded steps",
          "",
          ...report.managedRunState.steps.map(renderManagedRunStepLine),
          "",
        ]),
    "## Requested dependency specifiers",
    "",
    ...Object.entries(report.requestedDependencySpecifiers)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([packageName, specifier]) => `- ${packageName}: ${specifier}`),
    "",
    "## Installed packages",
    "",
    ...report.packages.map(renderInstalledPackageLine),
    "",
    "## Link wiring",
    "",
    ...report.links.map((item) => {
      const destination = item.resolvedPath ?? "<missing>";
      return `- ${renderStatus(item.status)} ${item.label}: ${destination}`;
    }),
    "",
    "## Test-root resolution",
    "",
    ...report.testRootResolutions.map(renderResolutionLine),
    "",
    "## Fixture checks",
    "",
    ...report.fixtures.flatMap((fixture) => [
      `### ${fixture.name}`,
      "",
      `- Status: ${renderStatus(fixture.status)} ${fixture.status}`,
      `- CWD: ${fixture.cwdPath}`,
      `- Required packages: ${fixture.requiredPackages.join(", ") || "(none)"}`,
      `- Missing packages: ${fixture.missingPackages.join(", ") || "(none)"}`,
      ...fixture.resolutions.map(renderResolutionLine),
      "",
    ]),
  ];

  if (report.npmLs.stdout || report.npmLs.stderr) {
    lines.push("## npm ls", "");
    if (report.npmLs.stdout) {
      lines.push("```json", report.npmLs.stdout.trim(), "```", "");
    }
    if (report.npmLs.stderr) {
      lines.push("```text", report.npmLs.stderr.trim(), "```", "");
    }
  }

  return `${lines.join("\n").replace(/\n{3,}/g, "\n\n")}\n`;
}

async function emitOutput(output: string, outputPath: string | null): Promise<void> {
  if (outputPath === null) {
    process.stdout.write(output);
    return;
  }

  await mkdir(pathDirname(outputPath), { recursive: true });
  await writeFile(outputPath, output);
  process.stdout.write(`Wrote real-package Svelte diagnostic report to ${outputPath}\n`);
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));
  const report = await buildReport(options.profileName, options.statePath);
  const output = options.format === "json" ? `${JSON.stringify(report, null, 2)}\n` : renderMarkdown(report);
  await emitOutput(output, options.outputPath);
}

await main();
