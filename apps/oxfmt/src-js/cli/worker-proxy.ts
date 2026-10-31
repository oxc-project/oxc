import { ChildProcess } from "node:child_process";
import Tinypool from "tinypool";
import { toFormatFileResult, toNullable } from "../libs/napi-callbacks";
import type { FormatFileResult } from "../libs/napi-callbacks";
import type {
  FormatFileParam,
  FormatEmbeddedDocParam,
  SortTailwindClassesArgs,
} from "../libs/apis";

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;
let poolSize: number | null = null;

export async function initExternalServices(numThreads: number): Promise<void> {
  // In LSP mode, this can be called repeatedly for the lifetime of the process.
  // e.g. on every workspace folder build, config-triggered rebuild, etc
  // The process-wide pool must never be recreated or destroyed on re-init:
  // that leaks the previous `child_process` workers, and other workspace folders may have formats in-flight.
  // (https://github.com/oxc-project/oxc/issues/24147)
  // NOTE: `numThreads` never changes within a single session, so the first value wins.
  poolSize ??= numThreads;
}

// Create the pool lazily on first use,
// so runs that never delegate to Prettier spawn no `child_process` workers at all.
// (e.g. Rust-tier files only)
async function getPool(): Promise<Tinypool> {
  // Rust always calls `initExternalServices` before formatting, so this is defensive.
  if (poolSize === null) throw new Error("External services are not initialized");

  pool ??= new Tinypool({
    filename: new URL("./cli-worker.js", import.meta.url).href,
    // Spawn workers on demand: each one is a full Node.js process,
    // and most runs delegate only a few files to Prettier.
    minThreads: 1,
    maxThreads: poolSize,
    // Keep spawned workers until the pool is destroyed.
    // Tinypool's default (0) removes workers above `minThreads` as soon as they go idle,
    // so a run whose tasks arrive in waves would keep respawning them.
    // `setTimeout` caps the delay at 2^31 - 1 ms; `Infinity` would fire after 1 ms.
    idleTimeout: 2 ** 31 - 1,
    // XXX: Use `child_process` instead of `worker_threads`.
    // Not sure why, but when using `worker_threads`,
    // calls from NAPI (CLI) -> worker threads -> NAPI (prettier-plugin-oxfmt) causes a hang...
    runtime: "child_process",
    // When setting the `runtime: child_process`,
    // `process.env` is not inherited (likely a bug), so it needs to be explicitly specified.
    env: process.env as Record<string, string>,
  });
  return pool;
}

export async function disposeExternalServices(): Promise<void> {
  await pool?.destroy();
  pool = null;
  poolSize = null;
}

// Exit the process without waiting for the workers to exit.
//
// `pool.destroy()` sends `SIGTERM` to each worker and waits for all of them to exit (~10 ms with 16 workers).
// There is nothing to wait for: workers write no files, their stdio is piped to this process,
// and Rust has written and flushed the output before `runCli()` resolves.
//
// `process.exit()` must follow the kills synchronously.
// If the event loop runs again, Tinypool sees a worker exit, emits "Worker exited unexpectedly" as an uncaught error,
// and may spawn a replacement worker.
//
// NOTE: This relies on a Tinypool internal that is not in its types:
// with `runtime: "child_process"`, each of `pool.threads` has the `ChildProcess` as `process`.
// If any worker does not, this returns without doing anything,
// and the caller must fall back to `disposeExternalServices()`.
export function exitWithoutWaitingForWorkers(exitCode: number): void {
  const children = (pool?.threads ?? []).map((worker) =>
    "process" in worker ? worker.process : undefined,
  );
  if (!children.every((child): child is ChildProcess => child instanceof ChildProcess)) return;

  for (const child of children) child.kill("SIGKILL");
  process.exit(exitCode);
}

// ---

export function formatFile(
  options: FormatFileParam["options"],
  code: string,
): Promise<FormatFileResult> {
  return toFormatFileResult(
    getPool().then((pool) =>
      pool.run({ options, code } satisfies FormatFileParam, { name: "formatFile" }),
    ),
  );
}

// ---

export function formatEmbeddedDoc(
  options: FormatEmbeddedDocParam["options"],
  code: string,
): Promise<string | null> {
  return toNullable(
    getPool().then((pool) =>
      pool.run({ options, code } satisfies FormatEmbeddedDocParam, {
        name: "formatEmbeddedDoc",
      }),
    ),
  );
}

export function sortTailwindClasses(
  options: SortTailwindClassesArgs["options"],
  classes: string[],
): Promise<string[] | null> {
  return toNullable(
    getPool().then((pool) =>
      pool.run({ classes, options } satisfies SortTailwindClassesArgs, {
        name: "sortTailwindClasses",
      }),
    ),
  );
}
