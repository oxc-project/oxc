import { parentPort, workerData } from "node:worker_threads";
import { registerWorker } from "../bindings.js";
import { loadPlugin } from "./load.ts";
import { lintFile, forgetBuffer, occupiedBufferCount } from "./lint.ts";
import { setupRuleConfigs } from "./config.ts";
import { createWorkspace, destroyWorkspace } from "../workspace/index.ts";
import { debugAssertIsNotUndefined } from "../utils/asserts.ts";

const { id } = workerData as { id: number };

const port = parentPort!;

/**
 * Drop the cached buffer, then tell the parent so tests can snapshot `occupiedBufferCount`.
 *
 * `OXLINT_TEST_OCCUPIED_BUFFERS` is a test-only hook. Production `forgetBuffer` stays a no-reply
 * cache null.
 */
function forgetBufferAndNotify(bufferId: number): void {
  forgetBuffer(bufferId);
  if (process.env.OXLINT_TEST_OCCUPIED_BUFFERS) {
    port.postMessage({ type: "forgot", id, count: occupiedBufferCount() });
  }
}

if (process.env.OXLINT_TEST_OCCUPIED_BUFFERS) {
  port.on("message", (message: { type: string }) => {
    if (message.type === "occupiedBufferCount") {
      port.postMessage({ type: "occupiedBufferCount", count: occupiedBufferCount() });
    }
  });
}

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
  forgetBuffer: process.env.OXLINT_TEST_OCCUPIED_BUFFERS ? forgetBufferAndNotify : forgetBuffer,
  setupRuleConfigs,
  // `JsCreateWorkspaceCb` is Promise-returning, matching `createWorkspaceWrapper` in `cli.ts`.
  createWorkspace: (workspaceUri) => Promise.resolve(createWorkspace(workspaceUri)),
  destroyWorkspace,
});

port.postMessage({ type: "ready", id });
