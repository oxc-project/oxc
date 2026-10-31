// Entry point for JS plugin workers (Node.js worker threads) which run JS plugins in parallel (`--js-threads` option).
//
// Registers this thread's JS plugin callbacks with Rust, then signals to main thread that it's ready.
// Rust then broadcasts plugin loading and config setup to this JS thread, and sends it files to lint.

import { parentPort } from "node:worker_threads";
import { registerJsPluginWorker } from "./bindings.js";
import {
  createWorkspaceWrapper,
  destroyWorkspaceWrapper,
  lintFileWrapper,
  loadPluginWrapper,
  setupRuleConfigsWrapper,
} from "./js_plugin_callbacks.ts";

registerJsPluginWorker(
  loadPluginWrapper,
  setupRuleConfigsWrapper,
  lintFileWrapper,
  createWorkspaceWrapper,
  destroyWorkspaceWrapper,
);

parentPort!.postMessage(null);
