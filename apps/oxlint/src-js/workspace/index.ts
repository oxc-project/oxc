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
  debugAssert(!workspaces.has(workspaceUri), `Workspace "${workspaceUri}" already exists`);

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
