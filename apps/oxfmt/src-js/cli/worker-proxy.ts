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
  // that leaks the previous workers, and other workspace folders may have formats in-flight.
  // (https://github.com/oxc-project/oxc/issues/24147)
  // NOTE: `numThreads` never changes within a single session, so the first value wins.
  poolSize ??= numThreads;
}

// Create the pool lazily on first use,
// so runs that never send a task to the pool spawn no workers at all.
// (e.g. Rust-tier files only, without Tailwind sorting)
async function getPool(): Promise<Tinypool> {
  // Rust always calls `initExternalServices` before formatting, so this is defensive.
  if (poolSize === null) throw new Error("External services are not initialized");

  // NOTE: Default `runtime: worker_threads` requires napi >= 3.16.1,
  // older versions deadlock when a worker loads the addon during `block_on`. (napi-rs/napi-rs#3670)
  pool ??= new Tinypool({
    filename: new URL("./cli-worker.js", import.meta.url).href,
    // Spawn workers on demand up to `poolSize`, and keep them alive for later tasks.
    // - Each worker loads its own copy of JS deps (e.g. Prettier, Tailwind sorter),
    //   and many runs send only a few tasks to the pool
    // - The default `idleTimeout: 0` removes idle workers above `minThreads` right away
    // - Unlike Piscina, Tinypool does not special-case `Infinity` and passes it to `setTimeout` as is,
    //   where Node.js replaces it with 1 ms, so use the max delay `setTimeout` accepts instead
    minThreads: 1,
    maxThreads: poolSize,
    idleTimeout: 2 ** 31 - 1,
  });
  return pool;
}

export async function disposeExternalServices(): Promise<void> {
  await pool?.destroy();
  pool = null;
  poolSize = null;
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
