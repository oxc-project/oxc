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

  const workspace = {
    cwd: "",
    allOptions: [],
    rules: [],
  };

  workspaces.set(workspaceUri, workspace);
  currentWorkspace = workspace;
  currentWorkspaceUri = workspaceUri;
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
