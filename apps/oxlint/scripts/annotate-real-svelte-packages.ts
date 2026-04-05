import { readFile } from "node:fs/promises";
import { isAbsolute as pathIsAbsolute, relative as pathRelative } from "node:path";
import process from "node:process";
import {
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  getRealSvelteDefaultReportPath,
  getRealSvelteDefaultRunStatePath,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";

import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";

interface CliOptions {
  profileName: RealSveltePackageProfileName;
  reportPath: string;
  statePath: string;
}

type AnnotationLevel = "error" | "warning" | "notice";

interface ManagedRunStepReport {
  stepName?: string;
  scriptName?: string;
  exitCode?: number | null;
  status?: string;
  errorMessage?: string | null;
}

interface ManagedRunStateReport {
  failureMessage?: string | null;
  laneStatus?: string;
  steps?: ManagedRunStepReport[];
}

interface PackageReport {
  expectedVersion?: string | null;
  installedVersion?: string | null;
  packageJsonPath?: string;
  packageName?: string;
  requestedSpecifier?: string;
  status?: string;
}

interface LinkReport {
  error?: string | null;
  label?: string;
  linkPath?: string;
  resolvedPath?: string | null;
  status?: string;
}

interface ResolutionReport {
  error?: string | null;
  insideInstallNodeModules?: boolean | null;
  packageName?: string;
  realPath?: string | null;
  scopeLabel?: string;
}

interface FixtureReport {
  missingPackages?: string[];
  name?: string;
  resolutions?: ResolutionReport[];
  status?: string;
}

interface RealSvelteReport {
  installPackageJsonMatches?: boolean;
  managedRunState?: ManagedRunStateReport | null;
  npmLs?: {
    exitCode?: number | null;
    stderr?: string;
  };
  packages?: PackageReport[];
  profileName?: string;
  fixtures?: FixtureReport[];
  links?: LinkReport[];
  testRootResolutions?: ResolutionReport[];
}

interface Annotation {
  file?: string;
  level: AnnotationLevel;
  message: string;
  title: string;
}

function parseCliOptions(argv: string[]): CliOptions {
  let profileName = resolveRealSveltePackageProfileName(undefined);
  let reportPath: string | null = null;
  let statePath: string | null = process.env.OXLINT_SVELTE_REAL_PACKAGES_STATE_PATH ?? null;

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

    if (arg === "--report-path") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("Expected a file path after --report-path.");
      }
      reportPath = value;
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

    if (arg === "--help") {
      process.stdout.write(
        `Usage: node --experimental-strip-types ./scripts/annotate-real-svelte-packages.ts [--profile <${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join("|")}>] [--report-path <path>] [--state-path <path>]\n`,
      );
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    profileName,
    reportPath: reportPath ?? getRealSvelteDefaultReportPath(profileName, "json"),
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

function escapeWorkflowValue(value: string): string {
  return value.replace(/%/g, "%25").replace(/\r/g, "%0D").replace(/\n/g, "%0A");
}

function escapePropertyValue(value: string): string {
  return escapeWorkflowValue(value).replace(/:/g, "%3A").replace(/,/g, "%2C");
}

function normalizeAnnotationFile(path: string | undefined): string | undefined {
  if (!path) {
    return undefined;
  }

  if (!pathIsAbsolute(path)) {
    return path;
  }

  return pathRelative(process.cwd(), path) || path;
}

function emitAnnotation(annotation: Annotation): void {
  const file = normalizeAnnotationFile(annotation.file);

  if (process.env.GITHUB_ACTIONS === "true") {
    const properties = [`title=${escapePropertyValue(annotation.title)}`];
    if (file) {
      properties.push(`file=${escapePropertyValue(file)}`);
    }
    process.stdout.write(
      `::${annotation.level} ${properties.join(",")}::${escapeWorkflowValue(annotation.message)}\n`,
    );
    return;
  }

  process.stdout.write(
    `[${annotation.level}] ${annotation.title}${file ? ` (${file})` : ""}: ${annotation.message}\n`,
  );
}

function collectAnnotations(
  report: RealSvelteReport | null,
  profileName: RealSveltePackageProfileName,
  reportPath: string,
  statePath: string,
): Annotation[] {
  if (report === null) {
    return [
      {
        level: "warning",
        title: "Oxlint Svelte diagnostics report missing",
        message:
          `No real-package Svelte diagnostics JSON was found for profile ${profileName}. ` +
          `Expected report: ${reportPath}. Expected state: ${statePath}.`,
      },
    ];
  }

  const annotations: Annotation[] = [];

  if (report.installPackageJsonMatches === false) {
    annotations.push({
      level: "error",
      title: "Oxlint Svelte helper manifest mismatch",
      message:
        `The installed helper package.json does not match the ${profileName} manifest. ` +
        `See ${reportPath} for details.`,
      file: reportPath,
    });
  }

  if (report.managedRunState?.laneStatus === "failed") {
    annotations.push({
      level: "error",
      title: "Oxlint Svelte managed lane failed",
      message: report.managedRunState.failureMessage ?? `Managed lane failed for profile ${profileName}.`,
      file: statePath,
    });
  }

  for (const step of report.managedRunState?.steps ?? []) {
    if (step.status !== "failed") {
      continue;
    }

    annotations.push({
      level: "error",
      title: `Oxlint Svelte step failed: ${step.stepName ?? step.scriptName ?? "unknown step"}`,
      message:
        step.errorMessage ??
        `${step.scriptName ?? "unknown command"} failed${typeof step.exitCode === "number" ? ` with exit code ${step.exitCode}` : ""}.`,
      file: statePath,
    });
  }

  for (const pkg of report.packages ?? []) {
    if (pkg.status === "ok") {
      continue;
    }

    const packageName = pkg.packageName ?? "unknown package";
    const expectedVersion = pkg.expectedVersion;
    const resolvedVersion = pkg.installedVersion ?? "<missing>";

    annotations.push({
      level: "error",
      title: `Oxlint Svelte package issue: ${packageName}`,
      message:
        expectedVersion === null || expectedVersion === undefined
          ? `${packageName} requested ${pkg.requestedSpecifier ?? "unknown specifier"}, resolved ${resolvedVersion}.`
          : `${packageName} expected ${expectedVersion}, resolved ${resolvedVersion}.`,
      file: pkg.packageJsonPath,
    });
  }

  for (const link of report.links ?? []) {
    if (link.status === "ok") {
      continue;
    }

    annotations.push({
      level: "error",
      title: `Oxlint Svelte link wiring issue: ${link.label ?? "unknown link"}`,
      message:
        link.error ??
        `Link status is ${link.status ?? "unknown"}; resolved target: ${link.resolvedPath ?? "<missing>"}.`,
      file: link.linkPath,
    });
  }

  for (const fixture of report.fixtures ?? []) {
    if (fixture.status !== "ok") {
      annotations.push({
        level: "error",
        title: `Oxlint Svelte fixture issue: ${fixture.name ?? "unknown fixture"}`,
        message:
          fixture.missingPackages && fixture.missingPackages.length > 0
            ? `Missing required packages: ${fixture.missingPackages.join(", ")}.`
            : `Fixture status is ${fixture.status ?? "unknown"}.`,
        file: reportPath,
      });
    }

    for (const resolution of fixture.resolutions ?? []) {
      if (resolution.error == null && resolution.insideInstallNodeModules !== false) {
        continue;
      }

      annotations.push({
        level: "error",
        title: `Oxlint Svelte resolution issue: ${resolution.scopeLabel ?? fixture.name ?? "unknown scope"}`,
        message:
          resolution.error ??
          `${resolution.packageName ?? "unknown package"} resolved outside the shared install root: ${resolution.realPath ?? "<unknown path>"}.`,
        file: reportPath,
      });
    }
  }

  for (const resolution of report.testRootResolutions ?? []) {
    if (resolution.error == null && resolution.insideInstallNodeModules !== false) {
      continue;
    }

    annotations.push({
      level: "error",
      title: `Oxlint Svelte test-root resolution issue: ${resolution.packageName ?? "unknown package"}`,
      message:
        resolution.error ??
        `${resolution.packageName ?? "unknown package"} resolved outside the shared install root: ${resolution.realPath ?? "<unknown path>"}.`,
      file: reportPath,
    });
  }

  if ((report.npmLs?.exitCode ?? 0) !== 0) {
    annotations.push({
      level: "warning",
      title: "Oxlint Svelte npm ls reported issues",
      message: report.npmLs?.stderr?.trim() || `npm ls exited with code ${report.npmLs?.exitCode ?? "unknown"}.`,
      file: reportPath,
    });
  }

  return annotations;
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));
  const report = (await safeReadJson(options.reportPath)) as RealSvelteReport | null;
  const annotations = collectAnnotations(report, options.profileName, options.reportPath, options.statePath);

  for (const annotation of annotations) {
    emitAnnotation(annotation);
  }

  process.stdout.write(
    `Emitted ${annotations.length} real-package Svelte annotation${annotations.length === 1 ? "" : "s"} for profile ${options.profileName}.\n`,
  );
}

await main();
