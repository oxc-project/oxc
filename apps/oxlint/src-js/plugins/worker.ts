import { parentPort, workerData } from "node:worker_threads";
import { registerWorker } from "../bindings.js";
import { loadPlugin } from "./load.ts";
import { lintFile, forgetBuffer } from "./lint.ts";
import { setupRuleConfigs } from "./config.ts";
import { createWorkspace, destroyWorkspace } from "../workspace/index.ts";
import { debugAssertIsNotUndefined } from "../utils/asserts.ts";

const { id } = workerData as { id: number };

// The callback types NAPI-RS generates for `Option<T>` arguments allow `undefined`, but `Option::None`
// on the Rust side arrives as `null`, never `undefined`. `cli.ts` narrows these the same way.
registerWorker({
  id,
  loadPlugin: (path, pluginName, pluginNameIsAlias, workspaceUri) => {
    debugAssertIsNotUndefined(pluginName, "`pluginName` should not be `undefined`");
    debugAssertIsNotUndefined(workspaceUri, "`workspaceUri` should not be `undefined`");
    return loadPlugin(path, pluginName, pluginNameIsAlias, workspaceUri);
  },
  lintFile: (
    filePath,
    bufferId,
    buffer,
    ruleIds,
    optionsIds,
    settingsJSON,
    globalsJSON,
    workspaceUri,
  ) => {
    debugAssertIsNotUndefined(buffer, "`buffer` should not be `undefined`");
    debugAssertIsNotUndefined(workspaceUri, "`workspaceUri` should not be `undefined`");
    return lintFile(
      filePath,
      bufferId,
      buffer,
      ruleIds,
      optionsIds,
      settingsJSON,
      globalsJSON,
      workspaceUri,
    );
  },
  forgetBuffer,
  setupRuleConfigs,
  // `JsCreateWorkspaceCb` is Promise-returning, matching `createWorkspaceWrapper` in `cli.ts`.
  createWorkspace: (workspaceUri) => Promise.resolve(createWorkspace(workspaceUri)),
  destroyWorkspace,
});

parentPort!.postMessage({ type: "ready", id });
