/*
 * Isolated Workspaces.
 *
 * Every workspace starts with a "workspace root" directory.
 *
 * Each workspace can be created, used, and then cleared to free up resources.
 */

import { debugAssert } from "../utils/asserts.ts";

import type { Options } from "../plugins/options.ts";
import type { RuleDetails } from "../plugins/load.ts";

/**
 * Settings for a workspace.
 */
interface Workspace {
  cwd: string;
  rules: RuleDetails[];
  allOptions: Readonly<Options>[];
}

/**
 * Active workspaces.
 * Keyed by workspace URI.
 */
export const workspaces = new Map<string, Workspace>();

/**
 * Most recent workspace that was used.
 */
export let currentWorkspace: Workspace | null = null;

/**
 * URI of most recent workspace that was used.
 */
export let currentWorkspaceUri: string | null = null;

/**
 * Create a new workspace.
 */
export function createWorkspace(workspaceUri: string): undefined {
  // This assertion is commented out because we currently don't destroy workspaces.
  // See comment in `destroyWorkspace` below.
  // debugAssert(!workspaces.has(workspaceUri), `Workspace "${workspaceUri}" already exists`);

  workspaces.set(workspaceUri, {
    cwd: "",
    allOptions: [],
    rules: [],
  });

  // Set current workspace to `null` to force switching workspace in the next call to `loadPlugin`.
  // Otherwise, if the new workspace has same URI as a previous workspace (which it does when reloading a workspace),
  // `cwd`, `registeredRules` and `allOptions` will still contain the state from the old version of the workspace.
  // This means `registeredRules` does not get replaced with an empty array before loading plugins again.
  // Forcing a switch to the new workspace overwrites the stale state.
  currentWorkspace = null;
  currentWorkspaceUri = null;
}

/**
 * Destroy a workspace.
 * Unloads all plugin data associated with this workspace.
 */
export function destroyWorkspace(workspaceUri: string): undefined {
  // We currently don't destroy workspaces.
  //
  // There is a race condition where sometimes LSP receives the signal to destroy a workspace *after* the signal
  // to create the new one when reloading a workspace.
  // Because both the old and the new workspaces have the same URI, the 2nd incarnation of the workspace would
  // be added to `workspaces`, but then it'd be removed again straight away when `destroyWorkspace` is called.
  //
  // It'd be ideal if we did remove `Workspace` objects to free up memory, but it's not *so* important for 2 reasons:
  //
  // 1. Usually workspace destruction only happens when you reload workspace, so it'll get discarded within seconds
  //    when `createWorkspace` over-writes it anyway.
  // 2. Most of the data in the workspace is persisted anyway in NodeJS module cache, so destroying workspace
  //    doesn't free that much memory.
  return;

  debugAssert(workspaces.has(workspaceUri), `Workspace "${workspaceUri}" does not exist`);

  workspaces.delete(workspaceUri);

  if (currentWorkspaceUri === workspaceUri) {
    currentWorkspace = null;
    currentWorkspaceUri = null;
  }
}

/**
 * Set the current workspace.
 * @param workspace - Workspace object
 * @param workspaceUri - Workspace URI
 */
export function setCurrentWorkspace(workspace: Workspace, workspaceUri: string): void {
  currentWorkspace = workspace;
  currentWorkspaceUri = workspaceUri;
}
