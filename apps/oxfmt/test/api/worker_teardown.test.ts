import { execa } from "execa";
import { describe, expect, it } from "vitest";

// Regression tests for the execute-after-unload crash (`0xC0000005` on Windows) fixed
// by `oxc_napi::pin_module_image` — see `crates/oxc_napi/src/module_pin.rs`.
// Each test runs in its own child process so a native crash fails the assertion
// instead of killing the vitest runner.

// Pre-fix, the crash reproduced within ~5 sequential teardowns on Windows.
const EXIT_ITERATIONS = 15;
// Same-process lifecycle: repeatedly drain to zero live environments (which triggers
// napi-rs's shared-runtime shutdown while the pinned addon stays loaded), then start
// the next batch to exercise runtime re-creation. Half of each batch is terminated
// mid-flight while the surviving workers keep formatting.
const CHURN_ROUNDS = 8;
const CHURN_BATCH = 6;

const distUrl = new URL("../../dist/index.js", import.meta.url).href;

// Worker code runs as ESM (`eval: true` inherits `--input-type=module`).
const formatWorker = `
import { parentPort } from "node:worker_threads";
const { format } = await import(${JSON.stringify(distUrl)});
const result = await format("test.ts", "let    x:number = 1;\\n");
parentPort.postMessage(result.code);
`;

// Starts a format call and reports back without awaiting it, so the parent can
// terminate the worker while the call is still in flight.
const midFlightWorker = `
import { parentPort } from "node:worker_threads";
const { format } = await import(${JSON.stringify(distUrl)});
void format("test.ts", "let    x:number = 1;\\n");
parentPort.postMessage("started");
`;

// The child sets `process.exitCode` and exits by draining naturally, so a native
// crash during worker teardown is not masked by an early `process.exit()`.
const formatAndExitChild = `
import { Worker } from "node:worker_threads";
const worker = new Worker(${JSON.stringify(formatWorker)}, { eval: true });
worker.once("error", (err) => {
  console.error(err);
  process.exitCode = 1;
});
worker.once("message", (msg) => {
  if (typeof msg !== "string" || !msg.includes("number")) process.exitCode = 1;
});
worker.once("exit", (code) => {
  if (code !== 0) process.exitCode = code;
});
`;

// One process, many rounds. The main thread never imports the binding, so between
// rounds the addon's live-environment count really reaches zero. `worker.terminate()`
// reports worker exit code 1 by design, so terminated workers' exit codes are ignored.
const sameProcessChurnChild = `
import { Worker } from "node:worker_threads";

function runWorker(midFlight) {
  const worker = new Worker(midFlight ? ${JSON.stringify(midFlightWorker)} : ${JSON.stringify(
    formatWorker,
  )}, { eval: true });
  return new Promise((resolve, reject) => {
    worker.once("error", reject);
    worker.once("message", (msg) => {
      if (midFlight) {
        void worker.terminate();
      } else if (typeof msg !== "string" || !msg.includes("number")) {
        reject(new Error("unexpected format output: " + msg));
      }
    });
    worker.once("exit", (code) => {
      if (!midFlight && code !== 0) reject(new Error("worker exited with " + code));
      else resolve();
    });
  });
}

for (let round = 0; round < ${CHURN_ROUNDS}; round++) {
  const batch = [];
  for (let i = 0; i < ${CHURN_BATCH}; i++) {
    batch.push(runWorker(i % 2 === 0));
  }
  await Promise.all(batch);
}
`;

async function runChild(source: string): Promise<void> {
  const result = await execa(process.execPath, ["--input-type=module", "-e", source], {
    reject: false,
    timeout: 120_000,
  });
  expect(result.stderr).toBe("");
  // On Windows an access violation surfaces as exit code 3221225477 (0xC0000005).
  expect(result.exitCode).toBe(0);
}

describe("worker_threads teardown", () => {
  it("survives repeated worker exits after format() resolves", { timeout: 120_000 }, async () => {
    await Promise.all(Array.from({ length: EXIT_ITERATIONS }, () => runChild(formatAndExitChild)));
  });

  it("survives same-process churn with mid-flight terminations", { timeout: 180_000 }, async () => {
    await runChild(sameProcessChurnChild);
  });
});
