import { setCwd } from "./context.ts";
import { setRegisteredRules } from "./load.ts";
import { setAllOptions } from "./options.ts";
import {
  workspaces,
  currentWorkspace,
  currentWorkspaceUri,
  setCurrentWorkspace,
} from "../workspace/index.ts";
import { debugAssertIsNonNull } from "../utils/asserts.ts";

export { currentWorkspace };

export function switchWorkspace(workspaceUri: string): void {
  // If requested workspace is already the current one, nothing to do.
  //
  // This is a fast path for common cases:
  // 1. Only a single workspace exists (most common).
  // 2. When user does have multiple workspaces, but works in one workspace for a lengthy period.
  if (currentWorkspaceUri === workspaceUri) return;

  // Get workspace
  const workspace = workspaces.get(workspaceUri);
  debugAssertIsNonNull(workspace, `Workspace "${workspaceUri}" does not exist`);

  // Change global state to that of the workspace
  setCwd(workspace.cwd);
  setRegisteredRules(workspace.rules);
  setAllOptions(workspace.allOptions);

  // Set this workspace as the current one
  setCurrentWorkspace(workspace, workspaceUri);
}
