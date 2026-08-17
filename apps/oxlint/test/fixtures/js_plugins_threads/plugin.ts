import { workerData } from "node:worker_threads";

import type { Plugin } from "#oxlint/plugins";

// Test-only: `cli.ts` puts a SharedArrayBuffer in workerData when
// `OXLINT_TEST_CREATE_ONCE_COUNTER` is set. Each worker isolate runs `createOnce` once.
const createOnceCounter = (workerData as { createOnceCounter?: SharedArrayBuffer } | undefined)
  ?.createOnceCounter;

const plugin: Plugin = {
  meta: { name: "threads-fixture" },
  rules: {
    "no-debugger": {
      createOnce(context) {
        if (createOnceCounter) {
          Atomics.add(new Int32Array(createOnceCounter), 0, 1);
        }
        return {
          DebuggerStatement(node) {
            context.report({ message: "debugger", node });
          },
        };
      },
    },
    "no-todo": {
      create(context) {
        return {
          Identifier(node) {
            if (node.name === "todo") context.report({ message: "todo", node });
          },
        };
      },
    },
  },
};

export default plugin;
