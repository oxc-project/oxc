/*
 * Isolated Workspaces.
 *
 * Every Workspace starts with a "workspace root" directory. This directory is
 * used to isolate the plugin's dependencies from other plugins and the main
 * application.
 *
 * Each workspace can be created, used, and then cleared to free up resources.
 *
 */

import { removePluginsInWorkspace, setupPluginSystemForWorkspace } from "../plugins/load";
import { removeOptionsInWorkspace, setupOptionsForWorkspace } from "../plugins/options";
import { debugAssert } from "../utils/asserts";

/**
 * Type representing a workspace identifier.
 * Currently, this is just a string representing the workspace root directory.
 */
export type WorkspaceIdentifier = string;
/**
 * Type representing a workspace.
 * Currently it only contains the identifier.
 */
export type Workspace = WorkspaceIdentifier;

/**
 * Set of workspace root directories.
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
 * Get a workspace by its identifier.
 */
export function getWorkspace(workspace: WorkspaceIdentifier): Workspace | null {
  return workspaces.has(workspace) ? workspace : null;
}

/**
 * Checks if a filePath is responsible for a workspace.
 */
export const isWorkspaceResponsible = (workspace: WorkspaceIdentifier, url: string): boolean => {
  return getResponsibleWorkspace(url) === workspace;
};

/**
 * Gets the workspace responsible for a given filePath.
 * Returns `null` if no workspace is responsible.
 *
 * This function is kept in sync with Rust's `WorkspaceWorker::is_responsible_for_file` (`oxc_language_server` crate).
 * Changing this function requires a corresponding change in Rust implementation.
 */
export const getResponsibleWorkspace = (filePath: string): Workspace | null => {
  return (
    [...workspaces.keys()]
      .filter((ws) => filePath.startsWith(ws))
      // Get the longest matching workspace path
      .sort((a, b) => b.length - a.length)[0] ?? null
  );
};
