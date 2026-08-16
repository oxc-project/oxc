import { parentPort, workerData } from "node:worker_threads";
import { registerWorker } from "../bindings.js";
import { loadPlugin } from "./load.ts";
import { lintFile, forgetBuffer } from "./lint.ts";
import { setupRuleConfigs } from "./config.ts";
import { createWorkspace, destroyWorkspace } from "../workspace/index.ts";

const { id } = workerData as { id: number };

registerWorker({
  id,
  loadPlugin,
  lintFile,
  forgetBuffer,
  setupRuleConfigs,
  // `JsCreateWorkspaceCb` is Promise-returning, matching `createWorkspaceWrapper` in `cli.ts`.
  createWorkspace: (workspaceUri) => Promise.resolve(createWorkspace(workspaceUri)),
  destroyWorkspace,
});

parentPort!.postMessage({ type: "ready", id });
