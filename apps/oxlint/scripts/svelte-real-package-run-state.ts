import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname as pathDirname } from "node:path";
import {
  getRealSvelteInstallRootPath,
} from "./svelte-real-package-metadata.ts";

import type { RealSveltePackageProfileName } from "./svelte-real-package-metadata.ts";

export type RealSvelteManagedRunReportMode = "always" | "failure" | "never";
export type RealSvelteManagedRunStatus = "running" | "passed" | "failed";

export interface RealSvelteManagedRunStepState {
  startedAt: string;
  finishedAt: string | null;
  scriptName: string;
  extraArgs: readonly string[];
  status: RealSvelteManagedRunStatus;
  stepName: string;
  errorMessage: string | null;
  exitCode: number | null;
}

export interface RealSvelteManagedRunState {
  build: boolean;
  createdAt: string;
  failureMessage: string | null;
  keepOnFailure: boolean;
  laneStatus: RealSvelteManagedRunStatus;
  profileName: RealSveltePackageProfileName;
  reportMode: RealSvelteManagedRunReportMode;
  requestedSuites: readonly string[];
  steps: RealSvelteManagedRunStepState[];
  updatedAt: string;
}

export interface CreateRealSvelteManagedRunStateOptions {
  build: boolean;
  keepOnFailure: boolean;
  profileName: RealSveltePackageProfileName;
  reportMode: RealSvelteManagedRunReportMode;
  requestedSuites: readonly string[];
}

export function getRealSvelteDefaultRunStatePath(profileName: RealSveltePackageProfileName): string {
  return `${getRealSvelteInstallRootPath(profileName)}-state.json`;
}

export async function readRealSvelteManagedRunState(
  statePath: string,
): Promise<RealSvelteManagedRunState | null> {
  try {
    return JSON.parse(await readFile(statePath, "utf8")) as RealSvelteManagedRunState;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return null;
    }

    throw error;
  }
}

export async function writeRealSvelteManagedRunState(
  statePath: string,
  state: RealSvelteManagedRunState,
): Promise<void> {
  await mkdir(pathDirname(statePath), { recursive: true });
  await writeFile(statePath, `${JSON.stringify(state, null, 2)}\n`);
}

export function createRealSvelteManagedRunState(
  options: CreateRealSvelteManagedRunStateOptions,
): RealSvelteManagedRunState {
  const now = new Date().toISOString();
  return {
    build: options.build,
    createdAt: now,
    failureMessage: null,
    keepOnFailure: options.keepOnFailure,
    laneStatus: "running",
    profileName: options.profileName,
    reportMode: options.reportMode,
    requestedSuites: [...options.requestedSuites],
    steps: [],
    updatedAt: now,
  };
}

export function getFailedRealSvelteManagedRunSteps(
  state: RealSvelteManagedRunState | null,
): RealSvelteManagedRunStepState[] {
  if (state === null) return [];
  return state.steps.filter((step) => step.status === "failed");
}
