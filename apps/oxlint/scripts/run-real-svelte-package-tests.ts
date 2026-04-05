import { spawnSync } from "node:child_process";
import { access, appendFile, readFile } from "node:fs/promises";
import process from "node:process";
import {
  PACKAGE_ROOT_PATH,
  REAL_SVELTE_MANAGED_SUITE_NAMES,
  REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES,
  REAL_SVELTE_PACKAGE_PROFILE_NAMES,
  getRealSvelteDefaultReportPath,
  resolveRealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";
import {
  createRealSvelteManagedRunState,
  getRealSvelteDefaultRunStatePath,
  writeRealSvelteManagedRunState,
} from "./svelte-real-package-run-state.ts";

import type {
  RealSvelteManagedSuiteName,
  RealSveltePackageProfileName,
} from "./svelte-real-package-metadata.ts";
import type {
  RealSvelteManagedRunReportMode,
  RealSvelteManagedRunState,
  RealSvelteManagedRunStatus,
} from "./svelte-real-package-run-state.ts";

const PNPM_COMMAND = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
const ALL_SUITE_NAMES = [...REAL_SVELTE_MANAGED_SUITE_NAMES];

type SuiteName = RealSvelteManagedSuiteName;

interface CliOptions {
  appendSummary: boolean;
  build: boolean;
  keepOnFailure: boolean;
  markdownReportPath: string;
  jsonReportPath: string;
  profileName: RealSveltePackageProfileName;
  reportMode: RealSvelteManagedRunReportMode;
  statePath: string;
  suiteNames: SuiteName[];
}

function parseReportMode(value: string | null | undefined): RealSvelteManagedRunReportMode {
  if (value === undefined || value === null || value === "") {
    return "failure";
  }
  if (value === "always" || value === "failure" || value === "never") {
    return value;
  }
  throw new Error(`Unknown report mode: ${value}. Expected one of: always, failure, never.`);
}

function parseCliOptions(argv: string[]): CliOptions {
  const suiteNames: SuiteName[] = [];
  let appendSummary = false;
  let build = false;
  let keepOnFailure = process.env.OXLINT_SVELTE_REAL_PACKAGES_KEEP_ON_FAILURE === "1";
  let profileName = resolveRealSveltePackageProfileName(undefined);
  let markdownReportPath: string | null = process.env.OXLINT_SVELTE_REAL_PACKAGES_REPORT_PATH ?? null;
  let jsonReportPath: string | null = null;
  let reportMode = parseReportMode(process.env.OXLINT_SVELTE_REAL_PACKAGES_REPORT_MODE);
  let statePath: string | null = null;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === "--") {
      continue;
    }

    if (arg === "--append-summary") {
      appendSummary = true;
      continue;
    }

    if (arg === "--build") {
      build = true;
      continue;
    }

    if (arg === "--keep-on-failure") {
      keepOnFailure = true;
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

    if (arg === "--report-mode") {
      const value = argv[index + 1];
      reportMode = parseReportMode(value);
      index += 1;
      continue;
    }

    if (arg === "--report-path" || arg === "--markdown-report-path") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error(`Expected a file path after ${arg}.`);
      }
      markdownReportPath = value;
      index += 1;
      continue;
    }

    if (arg === "--json-report-path") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("Expected a file path after --json-report-path.");
      }
      jsonReportPath = value;
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
        `Usage: node --experimental-strip-types ./scripts/run-real-svelte-package-tests.ts [runtime] [fixtures] [smoke] [lsp] [lsp-smoke] [--build] [--profile <${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join("|")}>] [--report-mode <always|failure|never>] [--markdown-report-path <path>] [--json-report-path <path>] [--state-path <path>] [--append-summary] [--keep-on-failure]\n`,
      );
      process.exit(0);
    }

    if (arg in REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES) {
      suiteNames.push(arg as SuiteName);
      continue;
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return {
    appendSummary,
    build,
    keepOnFailure,
    markdownReportPath: markdownReportPath ?? getRealSvelteDefaultReportPath(profileName, "markdown"),
    jsonReportPath: jsonReportPath ?? getRealSvelteDefaultReportPath(profileName, "json"),
    profileName,
    reportMode,
    statePath: statePath ?? getRealSvelteDefaultRunStatePath(profileName),
    suiteNames: suiteNames.length === 0 ? ALL_SUITE_NAMES : suiteNames,
  };
}

async function persistRunState(statePath: string, runState: RealSvelteManagedRunState): Promise<void> {
  runState.updatedAt = new Date().toISOString();
  await writeRealSvelteManagedRunState(statePath, runState);
}

function spawnPackageScript(
  scriptName: string,
  options: CliOptions,
  extraArgs: readonly string[] = [],
): number | null {
  process.stdout.write(
    `\n> pnpm run ${scriptName}${extraArgs.length === 0 ? "" : ` -- ${extraArgs.join(" ")}`}\n`,
  );

  const result = spawnSync(
    PNPM_COMMAND,
    ["run", scriptName, ...(extraArgs.length === 0 ? [] : ["--", ...extraArgs])],
    {
      cwd: PACKAGE_ROOT_PATH,
      stdio: "inherit",
      env: {
        ...process.env,
        OXLINT_SVELTE_REAL_PACKAGES_PROFILE: options.profileName,
        OXLINT_SVELTE_REAL_PACKAGES_REPORT_MODE: options.reportMode,
        OXLINT_SVELTE_REAL_PACKAGES_STATE_PATH: options.statePath,
      },
    },
  );

  return result.status;
}

async function runTrackedPackageScript(
  stepName: string,
  scriptName: string,
  options: CliOptions,
  runState: RealSvelteManagedRunState,
  extraArgs: readonly string[] = [],
): Promise<void> {
  const step = {
    startedAt: new Date().toISOString(),
    finishedAt: null,
    scriptName,
    extraArgs: [...extraArgs],
    status: "running",
    stepName,
    errorMessage: null,
    exitCode: null,
  } satisfies RealSvelteManagedRunState["steps"][number];

  runState.steps.push(step);
  await persistRunState(options.statePath, runState);

  const exitCode = spawnPackageScript(scriptName, options, extraArgs);
  step.exitCode = exitCode;
  step.finishedAt = new Date().toISOString();

  if (exitCode === 0) {
    step.status = "passed";
    await persistRunState(options.statePath, runState);
    return;
  }

  step.status = "failed";
  step.errorMessage = `Command failed: pnpm run ${scriptName} (exit code ${exitCode ?? "unknown"}).`;
  await persistRunState(options.statePath, runState);
  throw new Error(step.errorMessage);
}

function normalizeError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}

function shouldWriteReports(options: CliOptions, failure: Error | null): boolean {
  if (options.reportMode === "never") return false;
  if (options.reportMode === "always") return true;
  return failure !== null;
}

function setLaneStatus(
  runState: RealSvelteManagedRunState,
  status: RealSvelteManagedRunStatus,
  failureMessage: string | null,
): void {
  runState.laneStatus = status;
  runState.failureMessage = failureMessage;
}

async function appendMarkdownReportToSummary(markdownReportPath: string): Promise<void> {
  const summaryPath = process.env.GITHUB_STEP_SUMMARY;
  if (!summaryPath) {
    process.stdout.write("\nSkipping CI summary append because GITHUB_STEP_SUMMARY is not set.\n");
    return;
  }

  await access(markdownReportPath);
  const contents = await readFile(markdownReportPath, "utf8");
  await appendFile(summaryPath, `\n${contents}`);
  process.stdout.write(`\nAppended ${markdownReportPath} to $GITHUB_STEP_SUMMARY.\n`);
}

function generateReports(options: CliOptions): void {
  const reportCommands: Array<readonly string[]> = [
    ["--profile", options.profileName, "--format", "markdown", "--output", options.markdownReportPath],
    ["--profile", options.profileName, "--format", "json", "--output", options.jsonReportPath],
  ];

  for (const reportCommand of reportCommands) {
    const exitCode = spawnPackageScript("report:svelte-real-packages", options, reportCommand);
    if (exitCode !== 0) {
      throw new Error(
        `Command failed: pnpm run report:svelte-real-packages (exit code ${exitCode ?? "unknown"}).`,
      );
    }
  }
}

function cleanup(options: CliOptions, preserveDiagnostics: boolean): void {
  const exitCode = spawnPackageScript(
    "clean:svelte-real-packages",
    options,
    preserveDiagnostics ? ["--keep-diagnostics"] : [],
  );

  if (exitCode === 0) {
    return;
  }

  throw new Error(
    `Command failed: pnpm run clean:svelte-real-packages (exit code ${exitCode ?? "unknown"}).`,
  );
}

async function main(): Promise<void> {
  const options = parseCliOptions(process.argv.slice(2));
  const runState = createRealSvelteManagedRunState({
    build: options.build,
    keepOnFailure: options.keepOnFailure,
    profileName: options.profileName,
    reportMode: options.reportMode,
    requestedSuites: options.suiteNames,
  });
  await persistRunState(options.statePath, runState);

  let failure: Error | null = null;

  try {
    await runTrackedPackageScript("initial-clean", "clean:svelte-real-packages", options, runState);

    if (options.build) {
      await runTrackedPackageScript("build-test", "build-test", options, runState);
    }

    await runTrackedPackageScript(
      "setup",
      "setup:svelte-real-packages",
      options,
      runState,
      ["--profile", options.profileName],
    );
    await runTrackedPackageScript(
      "check",
      "check:svelte-real-packages",
      options,
      runState,
      ["--profile", options.profileName],
    );

    for (const suiteName of options.suiteNames) {
      await runTrackedPackageScript(
        suiteName,
        REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES[suiteName],
        options,
        runState,
      );
    }

    setLaneStatus(runState, "passed", null);
    await persistRunState(options.statePath, runState);
  } catch (error) {
    failure = normalizeError(error);
    setLaneStatus(runState, "failed", failure.message);
    await persistRunState(options.statePath, runState);
  }

  const writeReports = shouldWriteReports(options, failure);
  if (writeReports) {
    try {
      generateReports(options);
    } catch (reportError) {
      const normalizedReportError = normalizeError(reportError);
      failure =
        failure === null
          ? normalizedReportError
          : new Error(`${failure.message}\nReport generation also failed: ${normalizedReportError.message}`);
    }
  }

  if (writeReports && options.appendSummary) {
    try {
      await appendMarkdownReportToSummary(options.markdownReportPath);
    } catch (summaryError) {
      const normalizedSummaryError = normalizeError(summaryError);
      failure =
        failure === null
          ? normalizedSummaryError
          : new Error(`${failure.message}\nSummary append also failed: ${normalizedSummaryError.message}`);
    }
  }

  if (failure === null || !options.keepOnFailure) {
    try {
      cleanup(options, writeReports);
    } catch (cleanupError) {
      const normalizedCleanupError = normalizeError(cleanupError);
      failure =
        failure === null
          ? normalizedCleanupError
          : new Error(`${failure.message}\nCleanup also failed: ${normalizedCleanupError.message}`);
    }
  } else {
    process.stdout.write(
      `\nPreserved real-package Svelte helper state after failure. ` +
        `Profile: ${options.profileName}. Markdown report: ${options.markdownReportPath}. ` +
        `JSON report: ${options.jsonReportPath}. Run state: ${options.statePath}\n`,
    );
  }

  if (failure !== null) {
    throw failure;
  }

  process.stdout.write(
    `\nRan real-package Svelte suites: ${options.suiteNames.join(", ")}\n` +
      `Profile: ${options.profileName}\n` +
      `Run state: ${options.statePath}\n` +
      `${writeReports ? `Diagnostics: ${options.markdownReportPath}, ${options.jsonReportPath}\n` : ""}`,
  );
}

await main();
