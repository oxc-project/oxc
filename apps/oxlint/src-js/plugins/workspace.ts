/*
 * Isolated Workspaces for linting plugins.
 *
 * Every Workspace starts with a "workspace root" directory. This directory is
 * used to isolate the plugin's dependencies from other plugins and the main
 * application.
 *
 * Each workspace can be created, used, and then cleared to free up resources.
 */

import { registeredPluginUrls, registeredRules } from "./load.js";

/**
 * Set of workspace root directories.
 */
const workspaceRoots = new Set<string>();

/**
 * Creates a new workspace and initializes plugin and rule storage for it.
 *
 * Registers the workspace root directory and sets up empty collections for
 * plugin URLs and rules. Throws an error in DEBUG mode if the workspace
 * already exists.
 *
 * @param rootDir - The root directory of the workspace to create.
 */
export const createWorkspace = async (rootDir: string): Promise<undefined> => {
  if (DEBUG) {
    if (workspaceRoots.has(rootDir))
      throw new Error(`Workspace for rootDir "${rootDir}" already exists`);
  }

  workspaceRoots.add(rootDir);
  registeredPluginUrls.set(rootDir, new Set<string>());
  registeredRules.set(rootDir, []);
};

/**
 * Destroys an existing workspace and frees all associated resources.
 *
 * Removes the workspace root directory from the internal sets and maps,
 * effectively clearing all plugin URLs and rules registered for this workspace.
 * Throws an error in DEBUG mode if the workspace does not exist.
 *
 * @param rootDir - The root directory of the workspace to destroy.
 */
export const destroyWorkspace = (rootDir: string): undefined => {
  if (DEBUG) {
    if (!workspaceRoots.has(rootDir))
      throw new Error(`Workspace for rootDir "${rootDir}" does not exist`);
    if (!registeredPluginUrls.has(rootDir)) throw new Error("Invalid workspaceDir");
    if (!registeredRules.has(rootDir)) throw new Error("Invalid workspaceDir");
  }
  workspaceRoots.delete(rootDir);
  registeredPluginUrls.delete(rootDir);
  registeredRules.delete(rootDir);
};
