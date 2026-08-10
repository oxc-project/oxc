import { execa } from "execa";
import { describe, expect, it } from "vitest";

// Regression test for napi-rs's execute-after-unload crash (`0xC0000005` on Windows).
// Run in a child process so a native crash fails the assertion instead of killing Vitest.

// Pre-fix, the crash reproduced within ~5 sequential teardowns on Windows.
const EXIT_ITERATIONS = 15;
const distUrl = new URL("../../dist/index.js", import.meta.url).href;

// Worker code runs as ESM (`eval: true` inherits `--input-type=module`).
const formatWorker = `
import { parentPort } from "node:worker_threads";
const { format } = await import(${JSON.stringify(distUrl)});
const result = await format("test.ts", "let    x:number = 1;\\n");
parentPort.postMessage(result.code);
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

async function runChild(source: string) {
  return execa(process.execPath, ["--input-type=module", "-e", source], {
    reject: false,
    timeout: 120_000,
  });
}

describe("worker_threads teardown", () => {
  it("survives repeated worker exits after format() resolves", { timeout: 120_000 }, async () => {
    const results = await Promise.all(
      Array.from({ length: EXIT_ITERATIONS }, () => runChild(formatAndExitChild)),
    );
    for (const result of results) {
      expect(result.stderr).toBe("");
      // On Windows an access violation surfaces as exit code 3221225477 (0xC0000005).
      expect(result.exitCode).toBe(0);
    }
  });
});
