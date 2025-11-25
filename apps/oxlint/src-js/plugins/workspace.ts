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
 * Create a new workspace.
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
 * Destroy a workspace.
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
