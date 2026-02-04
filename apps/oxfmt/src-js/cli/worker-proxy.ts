import Tinypool from "tinypool";
import { resolvePlugins } from "../libs/prettier";
import type {
  FormatEmbeddedCodeParam,
  FormatFileParam,
  SortTailwindClassesArgs,
} from "../libs/prettier";
import type { Options } from "prettier";
import { SubprocessPool } from "./subprocess-pool";

// Node.js v24 has a race condition bug with worker_threads + ThreadsafeFunction.
// Fixed in Node.js v25.4.0+. See: https://github.com/nodejs/node/issues/55706
//
// Workaround: Use child_process pool instead of worker_threads on v24.
// Each child process has its own V8 isolate, avoiding the TSFN race condition.
const USE_SUBPROCESSES = !!process.env.OXFMT_USE_SUBPROCESSES;

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;
// Subprocess pool for Node.js v24 (avoids TSFN race condition)
let subprocessPool: SubprocessPool | null = null;

export async function initExternalFormatter(numThreads: number): Promise<string[]> {
  if (USE_SUBPROCESSES) {
    subprocessPool = new SubprocessPool(numThreads);
  } else {
    pool = new Tinypool({
      filename: new URL("./cli-worker.js", import.meta.url).href,
      minThreads: numThreads,
      maxThreads: numThreads,
    });
  }

  return resolvePlugins();
}

export async function disposeExternalFormatter(): Promise<void> {
  if (pool) {
    await pool.destroy();
    pool = null;
  }
  if (subprocessPool) {
    await subprocessPool.destroy();
    subprocessPool = null;
  }
}

export async function formatEmbeddedCode(
  options: Options,
  parserName: string,
  code: string,
): Promise<string> {
  if (USE_SUBPROCESSES) {
    return subprocessPool!.run("formatEmbeddedCode", {
      options,
      code,
      parserName,
    }) as Promise<string>;
  }
  return pool!.run({ options, code, parserName } satisfies FormatEmbeddedCodeParam, {
    name: "formatEmbeddedCode",
  });
}

export async function formatFile(
  options: Options,
  parserName: string,
  fileName: string,
  code: string,
): Promise<string> {
  if (USE_SUBPROCESSES) {
    return subprocessPool!.run("formatFile", {
      options,
      code,
      fileName,
      parserName,
    }) as Promise<string>;
  }
  return pool!.run({ options, code, fileName, parserName } satisfies FormatFileParam, {
    name: "formatFile",
  });
}

export async function sortTailwindClasses(
  filepath: string,
  options: Options,
  classes: string[],
): Promise<string[]> {
  if (USE_SUBPROCESSES) {
    return subprocessPool!.run("sortTailwindClasses", {
      filepath,
      options,
      classes,
    }) as Promise<string[]>;
  }
  return pool!.run({ filepath, options, classes } satisfies SortTailwindClassesArgs, {
    name: "sortTailwindClasses",
  });
}
