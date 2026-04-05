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
import { pathToFileURL } from "node:url";
import {
  PACKAGE_ROOT_PATH,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  REAL_SVELTE_REQUIRED_PACKAGES,
  REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH,
  REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
  REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH,
  getRealSvelteDefaultReportPath,
  getRealSvelteDependencySpecifierMap,
  getRealSvelteExpectedVersionMap,
  getRealSvelteInstallNodeModulesPath,
  getRealSvelteInstallPackageJsonPath,
  getRealSvelteInstallRootPath,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";
import {
  getFailedRealSvelteManagedRunSteps,
  getRealSvelteDefaultRunStatePath,
  readRealSvelteManagedRunState,
} from "./svelte-real-package-run-state.ts";

import type { RealSveltePackageProfileName, RealSvelteReportFormat } from "./svelte-real-package-metadata.ts";
import type { RealSvelteManagedRunState } from "./svelte-real-package-run-state.ts";

export interface CliOptions {
  format: RealSvelteReportFormat;
  outputPath: string | null;
  profileName: RealSveltePackageProfileName;
  statePath: string;
}

export interface PackageReport {
  packageName: string;
  requestedSpecifier: string;
  expectedVersion: string | null;
  installedVersion: string | null;
  status: "ok" | "missing" | "version-mismatch";
  packageJsonPath: string;
}

export interface LinkReport {
  label: string;
  linkPath: string;
  exists: boolean;
  isSymlink: boolean;
  resolvedPath: string | null;
  expectedTargetPath: string | null;
  status: "ok" | "missing" | "not-symlink" | "wrong-target";
  error: string | null;
}

export interface ResolutionReport {
  scopeLabel: string;
  fromDirPath: string;
  packageName: string;
  resolvedPath: string | null;
  realPath: string | null;
  insideInstallNodeModules: boolean | null;
  error: string | null;
}

export interface NpmLsReport {
  exitCode: number | null;
  stdout: string;
  stderr: string;
}

export interface RealSvelteReport {
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
  resolutions: ResolutionReport[];
  npmLs: NpmLsReport;
}

export function parseCliOptions(argv: string[]): CliOptions {
  let format: RealSvelteReportFormat = "markdown";
  let outputPath: string | null = process.env.OXFMT_SVELTE_REAL_PACKAGES_REPORT_PATH ?? null;
  let profileName = resolveRealSveltePackageProfileName(undefined);
  let statePath = process.env.OXFMT_SVELTE_REAL_PACKAGES_STATE_PATH ?? getRealSvelteDefaultRunStatePath(profileName);
  let useDefaultOutput = false;

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

    if (arg === "--profile") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("Expected a profile name after --profile.");
      }
      profileName = resolveRealSveltePackageProfileName(value);
      statePath = process.env.OXFMT_SVELTE_REAL_PACKAGES_STATE_PATH ?? getRealSvelteDefaultRunStatePath(profileName);
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

  return { format, outputPath, profileName, statePath };
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
  const packageJsonPath = getRealSvelteInstallPackageJsonPath(profileName);
  const packageJson = (await safeReadJson(packageJsonPath)) ?? {};
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

async function inspectInstalledPackages(profileName: RealSveltePackageProfileName): Promise<PackageReport[]> {
  const reports: PackageReport[] = [];
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);
  const expectedVersions = getRealSvelteExpectedVersionMap(profileName);

  for (const [packageName, requestedSpecifier] of Object.entries(getRealSvelteDependencySpecifierMap(profileName))) {
    const packageJsonPath = pathJoin(installNodeModulesPath, ...packageName.split("/"), "package.json");
    const packageJson = await safeReadJson(packageJsonPath);
    const installedVersion = typeof packageJson?.version === "string" ? packageJson.version : null;
    const expectedVersion = expectedVersions[packageName] ?? null;

    reports.push({
      packageName,
      requestedSpecifier,
      expectedVersion,
      installedVersion,
      status:
        installedVersion === null
          ? "missing"
          : expectedVersion !== null && installedVersion !== expectedVersion
            ? "version-mismatch"
            : "ok",
      packageJsonPath,
    });
  }

  return reports;
}

async function inspectLink(linkPath: string, expectedTargetPath: string, label: string): Promise<LinkReport> {
  try {
    const stats = await lstat(linkPath);
    const resolvedPath = await safeRealpath(linkPath);
    const expectedResolvedPath = await safeRealpath(expectedTargetPath);
    const isSymlink = stats.isSymbolicLink() || process.platform === "win32";

    return {
      label,
      linkPath,
      exists: true,
      isSymlink,
      resolvedPath,
      expectedTargetPath: expectedResolvedPath,
      status: !isSymlink
        ? "not-symlink"
        : resolvedPath !== null && expectedResolvedPath !== null && resolvedPath !== expectedResolvedPath
          ? "wrong-target"
          : "ok",
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
        expectedTargetPath: await safeRealpath(expectedTargetPath),
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
      expectedTargetPath: await safeRealpath(expectedTargetPath),
      status: "missing",
      error: (error as Error).message,
    };
  }
}

async function inspectPackageResolution(
  scopeLabel: string,
  fromDirPath: string,
  anchorPath: string,
  packageName: string,
  installNodeModulesPath: string,
): Promise<ResolutionReport> {
  try {
    const requireFromDir = createRequire(anchorPath);
    const resolvedPath = requireFromDir.resolve(packageName);
    const realPath = await safeRealpath(resolvedPath);

    return {
      scopeLabel,
      fromDirPath,
      packageName,
      resolvedPath,
      realPath,
      insideInstallNodeModules: realPath === null ? null : isPathInside(installNodeModulesPath, realPath),
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
      error: (error as Error).message,
    };
  }
}

async function inspectResolutions(profileName: RealSveltePackageProfileName): Promise<ResolutionReport[]> {
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(profileName);
  const scopes = [
    {
      scopeLabel: "runtime fixture",
      fromDirPath: REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH,
      anchorPath: pathJoin(REAL_SVELTE_RUNTIME_FIXTURE_DIR_PATH, "empty.json"),
    },
    {
      scopeLabel: "runtime config dir",
      fromDirPath: REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH,
      anchorPath: pathJoin(REAL_SVELTE_RUNTIME_CONFIG_DIR_PATH, ".oxfmtrc.json"),
    },
  ] as const;

  const reports: ResolutionReport[] = [];

  for (const scope of scopes) {
    for (const packageName of REAL_SVELTE_REQUIRED_PACKAGES) {
      reports.push(
        await inspectPackageResolution(
          scope.scopeLabel,
          scope.fromDirPath,
          scope.anchorPath,
          packageName,
          installNodeModulesPath,
        ),
      );
    }
  }

  return reports;
}

function runNpmLs(profileName: RealSveltePackageProfileName): NpmLsReport {
  const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
  const result = spawnSync(npmCommand, ["ls", "--all", "--json"], {
    cwd: getRealSvelteInstallRootPath(profileName),
    encoding: "utf8",
  });

  return {
    exitCode: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

export async function buildReport(options: CliOptions): Promise<RealSvelteReport> {
  const installRootPath = getRealSvelteInstallRootPath(options.profileName);
  const installNodeModulesPath = getRealSvelteInstallNodeModulesPath(options.profileName);
  const installPackageJsonPath = getRealSvelteInstallPackageJsonPath(options.profileName);
  const installPackageJson = await inspectInstallPackageJson(options.profileName);
  const managedRunState = await readRealSvelteManagedRunState(options.statePath);

  return {
    generatedAt: new Date().toISOString(),
    profileName: options.profileName,
    platform: process.platform,
    nodeVersion: process.version,
    packageRootPath: PACKAGE_ROOT_PATH,
    installRootPath,
    installNodeModulesPath,
    installPackageJsonPath,
    installPackageJsonMatches: installPackageJson.installPackageJsonMatches,
    managedRunState,
    managedRunStatePath: options.statePath,
    installPackageJsonDependencies: installPackageJson.dependencies,
    requestedDependencySpecifiers: { ...getRealSvelteDependencySpecifierMap(options.profileName) },
    packages: await inspectInstalledPackages(options.profileName),
    links: [
      await inspectLink(
        REAL_SVELTE_RUNTIME_FIXTURE_NODE_MODULES_PATH,
        installNodeModulesPath,
        "runtime fixture node_modules",
      ),
    ],
    resolutions: await inspectResolutions(options.profileName),
    npmLs: runNpmLs(options.profileName),
  };
}

function renderPackageStatusLine(pkg: PackageReport): string {
  const expected = pkg.expectedVersion === null ? "(floating)" : pkg.expectedVersion;
  const installed = pkg.installedVersion ?? "missing";
  return `- ${pkg.packageName}: ${pkg.status} (requested: ${pkg.requestedSpecifier}; expected: ${expected}; installed: ${installed})`;
}

function renderResolutionStatusLine(resolution: ResolutionReport): string {
  return `- [${resolution.scopeLabel}] ${resolution.packageName}: ${resolution.error === null ? `ok (${resolution.realPath ?? resolution.resolvedPath ?? "unknown"})` : `error (${resolution.error})`}`;
}

export function renderMarkdownReport(report: RealSvelteReport): string {
  const failedSteps = getFailedRealSvelteManagedRunSteps(report.managedRunState);
  const lines = [
    `# Oxfmt real-package Svelte report (${report.profileName})`,
    "",
    `- Generated at: ${report.generatedAt}`,
    `- Platform: ${report.platform}`,
    `- Node: ${report.nodeVersion}`,
    `- Install root: \`${report.installRootPath}\``,
    `- Install package.json matches requested dependencies: ${report.installPackageJsonMatches ? "yes" : "no"}`,
    `- Managed lane status: ${report.managedRunState?.laneStatus ?? "unknown"}`,
    `- Failed managed steps: ${failedSteps.length}`,
    "",
    "## Packages",
    ...report.packages.map(renderPackageStatusLine),
    "",
    "## Links",
    ...report.links.map((link) =>
      `- ${link.label}: ${link.status}${link.resolvedPath ? ` -> ${link.resolvedPath}` : ""}${link.error ? ` (${link.error})` : ""}`),
    "",
    "## Resolutions",
    ...report.resolutions.map(renderResolutionStatusLine),
    "",
    "## npm ls",
    `- Exit code: ${report.npmLs.exitCode ?? "unknown"}`,
  ];

  if (report.npmLs.stderr.trim() !== "") {
    lines.push("", "```text", report.npmLs.stderr.trim(), "```");
  }

  if (failedSteps.length > 0) {
    lines.push("", "## Failed managed steps");
    for (const step of failedSteps) {
      lines.push(`- ${step.stepName}: ${step.errorMessage ?? `exit code ${step.exitCode ?? "unknown"}`}`);
    }
  }

  return `${lines.join("\n").trimEnd()}\n`;
}

export async function writeOutput(outputPath: string | null, content: string): Promise<void> {
  if (outputPath === null) {
    process.stdout.write(content);
    return;
  }

  await mkdir(pathDirname(outputPath), { recursive: true });
  await writeFile(outputPath, content);
  process.stdout.write(`Wrote ${outputPath}.\n`);
}

export async function main(argv: string[] = process.argv.slice(2)): Promise<void> {
  const options = parseCliOptions(argv);
  const report = await buildReport(options);
  const content = options.format === "json"
    ? `${JSON.stringify(report, null, 2)}\n`
    : renderMarkdownReport(report);

  await writeOutput(options.outputPath, content);
}


function isExecutedAsScript(): boolean {
  return process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href;
}

if (isExecutedAsScript()) {
  await main();
}
