/*
 * Isolated Workspaces.
 *
 * Every Workspace starts with a "workspace root" directory. This directory is
 * used to isolate the plugin's dependencies from other plugins and the main
 * application.
 *
 * Each workspace can be created, used, and then cleared to free up resources.
 */

import { removePluginsInWorkspace, setupPluginSystemForWorkspace } from "../plugins/load";
import { removeOptionsInWorkspace, setupOptionsForWorkspace } from "../plugins/options";
import { debugAssert } from "../utils/asserts";

/**
 * Type representing a workspace ID.
 * Currently, this is just a string representing the workspace root directory as `file://` URL.
 */
export type WorkspaceIdentifier = string;

/**
 * Type representing a workspace.
 * Currently it only contains the workspace root directory as `file://` URL.
 */
export type Workspace = WorkspaceIdentifier;

/**
 * Set of workspace IDs.
 */
const workspaces = new Set<Workspace>();

/**
 * Create a new workspace.
 */
export function createWorkspace(workspace: WorkspaceIdentifier): undefined {
  debugAssert(!workspaces.has(workspace), `Workspace "${workspace.toString()}" already exists`);
  workspaces.add(workspace);
  setupPluginSystemForWorkspace(workspace);
  setupOptionsForWorkspace(workspace);
}

/**
 * Destroy a workspace.
 * Unloads all plugin data associated with this workspace.
 */
export function destroyWorkspace(workspace: WorkspaceIdentifier): undefined {
  debugAssert(workspaces.has(workspace), `Workspace "${workspace.toString()}" does not exist`);

  workspaces.delete(workspace);
  removePluginsInWorkspace(workspace);
  removeOptionsInWorkspace(workspace);
}

/**
 * Gets the CLI workspace ID.
 * In CLI mode, there is exactly one workspace (the CWD), so this returns that workspace ID.
 */
export function getCliWorkspace(): Workspace {
  debugAssert(
    workspaces.size === 1,
    "getCliWorkspace should only be used in CLI mode with 1 workspace",
  );
  return workspaces.values().next().value;
}
