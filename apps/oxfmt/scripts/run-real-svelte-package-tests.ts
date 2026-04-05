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
  let keepOnFailure = process.env.OXFMT_SVELTE_REAL_PACKAGES_KEEP_ON_FAILURE === "1";
  let profileName = resolveRealSveltePackageProfileName(undefined);
  let markdownReportPath: string | null = process.env.OXFMT_SVELTE_REAL_PACKAGES_REPORT_PATH ?? null;
  let jsonReportPath: string | null = null;
  let reportMode = parseReportMode(process.env.OXFMT_SVELTE_REAL_PACKAGES_REPORT_MODE);
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
        `Usage: node --experimental-strip-types ./scripts/run-real-svelte-package-tests.ts [${ALL_SUITE_NAMES.join("] [")}] [--build] [--profile <${REAL_SVELTE_PACKAGE_PROFILE_NAMES.join("|")}>] [--report-mode <always|failure|never>] [--markdown-report-path <path>] [--json-report-path <path>] [--state-path <path>] [--append-summary] [--keep-on-failure]\n`,
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
        OXFMT_SVELTE_REAL_PACKAGES_CI: "1",
        OXFMT_SVELTE_REAL_PACKAGES_PROFILE: options.profileName,
        OXFMT_SVELTE_REAL_PACKAGES_REPORT_MODE: options.reportMode,
        OXFMT_SVELTE_REAL_PACKAGES_STATE_PATH: options.statePath,
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
  const sharedArgs = ["--profile", options.profileName, "--state-path", options.statePath] as const;
  const commands = [
    {
      format: "markdown",
      outputPath: options.markdownReportPath,
    },
    {
      format: "json",
      outputPath: options.jsonReportPath,
    },
  ] as const;

  for (const command of commands) {
    const exitCode = spawnPackageScript(
      "report:svelte-real-packages",
      options,
      [...sharedArgs, "--format", command.format, "--output", command.outputPath],
    );

    if (exitCode !== 0) {
      throw new Error(`Failed to generate ${command.format} real-package Svelte formatter report.`);
    }
  }
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
  let failure: Error | null = null;

  await persistRunState(options.statePath, runState);

  try {
    await runTrackedPackageScript("initial clean", "clean:svelte-real-packages", options, runState, [
      "--profile",
      options.profileName,
    ]);

    if (options.build) {
      await runTrackedPackageScript("build", "build-test", options, runState);
    }

    await runTrackedPackageScript("setup", "setup:svelte-real-packages", options, runState, [
      "--profile",
      options.profileName,
    ]);
    await runTrackedPackageScript("check", "check:svelte-real-packages", options, runState, [
      "--profile",
      options.profileName,
    ]);

    for (const suiteName of options.suiteNames) {
      await runTrackedPackageScript(suiteName, REAL_SVELTE_MANAGED_SUITE_SCRIPT_NAMES[suiteName], options, runState);
    }

    setLaneStatus(runState, "passed", null);
  } catch (error) {
    failure = normalizeError(error);
    setLaneStatus(runState, "failed", failure.message);
  } finally {
    await persistRunState(options.statePath, runState);

    const writeReports = shouldWriteReports(options, failure);
    if (writeReports) {
      generateReports(options);
      if (options.appendSummary) {
        await appendMarkdownReportToSummary(options.markdownReportPath);
      }
    }

    if (failure && options.keepOnFailure) {
      process.stdout.write(
        `\nPreserving real-package Svelte formatter helper state for profile ${options.profileName}.\n`,
      );
    } else {
      const exitCode = spawnPackageScript(
        "clean:svelte-real-packages",
        options,
        ["--profile", options.profileName, ...(writeReports ? ["--keep-diagnostics"] : [])],
      );
      if (exitCode !== 0) {
        throw new Error(`Command failed: pnpm run clean:svelte-real-packages (exit code ${exitCode ?? "unknown"}).`);
      }
    }
  }

  if (failure) {
    throw failure;
  }
}

await main();
