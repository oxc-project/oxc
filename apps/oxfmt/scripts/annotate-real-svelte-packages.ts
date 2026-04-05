import { readFile } from "node:fs/promises";
import { isAbsolute as pathIsAbsolute, relative as pathRelative } from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import {
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  getRealSvelteDefaultReportPath,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";
import { getRealSvelteDefaultRunStatePath } from "./svelte-real-package-run-state.ts";

import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";

export interface CliOptions {
  profileName: RealSveltePackageProfileName;
  reportPath: string;
  statePath: string;
}

export type AnnotationLevel = "error" | "warning" | "notice";

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
  packageName?: string;
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

interface RealSvelteReport {
  installPackageJsonMatches?: boolean;
  managedRunState?: ManagedRunStateReport | null;
  npmLs?: {
    exitCode?: number | null;
    stderr?: string;
  };
  packages?: PackageReport[];
  profileName?: string;
  links?: LinkReport[];
  resolutions?: ResolutionReport[];
}

export interface Annotation {
  file?: string;
  level: AnnotationLevel;
  message: string;
  title: string;
}

export function parseCliOptions(argv: string[]): CliOptions {
  let profileName = resolveRealSveltePackageProfileName(undefined);
  let reportPath: string | null = null;
  let statePath: string | null = process.env.OXFMT_SVELTE_REAL_PACKAGES_STATE_PATH ?? null;

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

export function escapeWorkflowValue(value: string): string {
  return value.replace(/%/g, "%25").replace(/\r/g, "%0D").replace(/\n/g, "%0A");
}

export function escapePropertyValue(value: string): string {
  return escapeWorkflowValue(value).replace(/:/g, "%3A").replace(/,/g, "%2C");
}

export function normalizeAnnotationFile(path: string | undefined): string | undefined {
  if (!path) {
    return undefined;
  }

  if (!pathIsAbsolute(path)) {
    return path;
  }

  return pathRelative(process.cwd(), path) || path;
}

export function emitAnnotation(annotation: Annotation): void {
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

export function collectAnnotations(
  report: RealSvelteReport | null,
  profileName: RealSveltePackageProfileName,
  reportPath: string,
  statePath: string,
): Annotation[] {
  if (report === null) {
    return [
      {
        level: "warning",
        title: "Oxfmt Svelte diagnostics report missing",
        message:
          `No real-package Svelte formatter diagnostics JSON was found for profile ${profileName}. ` +
          `Expected report: ${reportPath}. Expected state: ${statePath}.`,
      },
    ];
  }

  const annotations: Annotation[] = [];

  if (report.installPackageJsonMatches === false) {
    annotations.push({
      level: "error",
      title: "Oxfmt Svelte helper manifest mismatch",
      message:
        `The installed helper package.json does not match the ${profileName} manifest. ` +
        `See ${reportPath} for details.`,
      file: reportPath,
    });
  }

  if (report.managedRunState?.laneStatus === "failed") {
    annotations.push({
      level: "error",
      title: "Oxfmt Svelte managed lane failed",
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
      title: `Oxfmt Svelte step failed: ${step.stepName ?? step.scriptName ?? "unknown step"}`,
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
      title: `Oxfmt Svelte package issue: ${packageName}`,
      message:
        pkg.status === "version-mismatch"
          ? `Expected ${packageName}@${expectedVersion ?? "<floating>"} but found ${resolvedVersion}.`
          : `Missing required package ${packageName}.`,
      file: reportPath,
    });
  }

  for (const link of report.links ?? []) {
    if (link.status === "ok") {
      continue;
    }

    annotations.push({
      level: "error",
      title: `Oxfmt Svelte link issue: ${link.label ?? "unknown link"}`,
      message: link.error ?? `${link.label ?? "unknown link"} is ${link.status ?? "invalid"}.`,
      file: link.linkPath ?? reportPath,
    });
  }

  for (const resolution of report.resolutions ?? []) {
    if (resolution.error) {
      annotations.push({
        level: "error",
        title: `Oxfmt Svelte resolution error: ${resolution.packageName ?? "unknown package"}`,
        message: resolution.error,
        file: reportPath,
      });
      continue;
    }

    if (resolution.insideInstallNodeModules === false) {
      annotations.push({
        level: "warning",
        title: `Oxfmt Svelte resolution drift: ${resolution.packageName ?? "unknown package"}`,
        message:
          `${resolution.packageName ?? "unknown package"} resolved outside the helper install in ` +
          `${resolution.scopeLabel ?? "unknown scope"}.`,
        file: reportPath,
      });
    }
  }

  if ((report.npmLs?.exitCode ?? 0) !== 0) {
    annotations.push({
      level: "warning",
      title: "Oxfmt Svelte npm ls reported problems",
      message: report.npmLs?.stderr?.trim() || `npm ls exited with code ${report.npmLs?.exitCode ?? "unknown"}.`,
      file: reportPath,
    });
  }

  return annotations;
}

export async function main(argv: string[] = process.argv.slice(2)): Promise<void> {
  const options = parseCliOptions(argv);
  const report = await safeReadJson(options.reportPath) as RealSvelteReport | null;
  const annotations = collectAnnotations(report, options.profileName, options.reportPath, options.statePath);

  if (annotations.length === 0) {
    emitAnnotation({
      level: "notice",
      title: "Oxfmt Svelte diagnostics clean",
      message: `No real-package Svelte formatter issues found for profile ${options.profileName}.`,
      file: options.reportPath,
    });
    return;
  }

  for (const annotation of annotations) {
    emitAnnotation(annotation);
  }
}


function isExecutedAsScript(): boolean {
  return process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href;
}

if (isExecutedAsScript()) {
  await main();
}
