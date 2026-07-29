import { execa } from "execa";
import { describe, expect, it } from "vitest";

// Regression test for the execute-after-unload crash (`0xC0000005` on Windows) fixed by
// `oxc_napi::pin_module_image` — see `crates/oxc_napi/src/module_pin.rs`.
// Each iteration runs in its own child process so a native crash fails the assertion
// instead of killing the vitest runner; children are independent, so they run
// concurrently without weakening the reproduction.

// Pre-fix, the crash reproduced within ~5 sequential teardowns on Windows.
const EXIT_ITERATIONS = 15;
const TERMINATE_ITERATIONS = 8;

const distUrl = new URL("../../dist/index.js", import.meta.url).href;

// Worker code runs as ESM (`eval: true` inherits `--input-type=module`).
// The child sets `process.exitCode` and exits by draining naturally, so a native crash
// during worker teardown is not masked by an early `process.exit()`.
const formatAndExitChild = `
import { Worker } from "node:worker_threads";
const worker = new Worker(${JSON.stringify(`
import { parentPort } from "node:worker_threads";
const { format } = await import(${JSON.stringify(distUrl)});
const result = await format("test.ts", "let    x:number = 1;\\n");
parentPort.postMessage(result.code);
`)}, { eval: true });
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

// Terminates the worker while a format() call is still in flight. The worker's own exit
// code is ignored: `worker.terminate()` reports exit code 1 by design.
const midFlightTerminateChild = `
import { Worker } from "node:worker_threads";
const worker = new Worker(${JSON.stringify(`
import { parentPort } from "node:worker_threads";
const { format } = await import(${JSON.stringify(distUrl)});
void format("test.ts", "let    x:number = 1;\\n");
parentPort.postMessage("started");
`)}, { eval: true });
worker.once("error", (err) => {
  console.error(err);
  process.exitCode = 1;
});
worker.once("message", () => {
  void worker.terminate();
});
`;

async function runChild(source: string): Promise<void> {
  const result = await execa("node", ["--input-type=module", "-e", source], {
    reject: false,
    timeout: 60_000,
  });
  expect(result.stderr).toBe("");
  // On Windows an access violation surfaces as exit code 3221225477 (0xC0000005).
  expect(result.exitCode).toBe(0);
}

describe("worker_threads teardown", () => {
  it("survives repeated worker exits after format() resolves", { timeout: 120_000 }, async () => {
    await Promise.all(Array.from({ length: EXIT_ITERATIONS }, () => runChild(formatAndExitChild)));
  });

  it("survives worker.terminate() while format() is in flight", { timeout: 120_000 }, async () => {
    await Promise.all(
      Array.from({ length: TERMINATE_ITERATIONS }, () => runChild(midFlightTerminateChild)),
    );
  });
});
