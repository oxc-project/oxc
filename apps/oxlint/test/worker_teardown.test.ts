import { execa } from "execa";
import { describe, expect, it } from "vitest";

// Regression test for napi-rs's execute-after-unload crash (`0xC0000005` on Windows).
// Oxlint's addon runs on napi-rs's Tokio runtime, whose code can outlive a worker.
// Loading the binding in a worker starts the runtime; worker exit tears the environment
// down. The main thread never imports the binding, so between rounds the addon's
// live-environment count really reaches zero and exercises runtime teardown.
// Runs in a child process so a native crash fails the assertion instead of the runner.

const ROUNDS = 5;

const bindingsUrl = new URL("../src-js/bindings.js", import.meta.url).href;

const workerSource = `
import { parentPort } from "node:worker_threads";
await import(${JSON.stringify(bindingsUrl)});
parentPort.postMessage("loaded");
`;

const childSource = `
import { Worker } from "node:worker_threads";

for (let round = 0; round < ${ROUNDS}; round++) {
  await new Promise((resolve, reject) => {
    const worker = new Worker(${JSON.stringify(workerSource)}, { eval: true });
    worker.once("error", reject);
    worker.once("exit", (code) => {
      if (code !== 0) reject(new Error("worker exited with " + code));
      else resolve();
    });
  });
}
`;

describe("worker_threads teardown", () => {
  it("survives repeated binding load/unload in workers", { timeout: 120_000 }, async () => {
    const result = await execa(process.execPath, ["--input-type=module", "-e", childSource], {
      reject: false,
      timeout: 60_000,
    });
    expect(result.stderr).toBe("");
    // On Windows an access violation surfaces as exit code 3221225477 (0xC0000005).
    expect(result.exitCode).toBe(0);
  });
});
