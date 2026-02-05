import Tinypool from "tinypool";
import { resolvePlugins } from "../libs/prettier";
import type {
  FormatEmbeddedCodeParam,
  FormatFileParam,
  SortTailwindClassesArgs,
} from "../libs/prettier";
import type { Options } from "prettier";

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;

export async function initExternalFormatter(numThreads: number): Promise<string[]> {
  pool = new Tinypool({
    filename: new URL("./cli-worker.js", import.meta.url).href,
    minThreads: numThreads,
    maxThreads: numThreads,
    // NOTE: Node.js has a race condition bug with `ThreadsafeFunction`
    // that causes V8 crashes when using `worker_threads`.
    // See also https://github.com/nodejs/node/issues/55706
    //
    // This is fixed in v25.4.0+,
    // so we can switch back to `worker_threads` by checking Node.js version later.
    // However,
    // - to make behavior consistent across different versions
    // - and performance impact is negligible
    // we continue to use `child_process` for now.
    runtime: "child_process",
  });

  return resolvePlugins();
}

export async function disposeExternalFormatter(): Promise<void> {
  await pool?.destroy();
  pool = null;
}

export async function formatEmbeddedCode(
  options: Options,
  parserName: string,
  code: string,
): Promise<string> {
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
  return pool!.run({ options, code, fileName, parserName } satisfies FormatFileParam, {
    name: "formatFile",
  });
}

export async function sortTailwindClasses(
  filepath: string,
  options: Options,
  classes: string[],
): Promise<string[]> {
  return pool!.run({ filepath, options, classes } satisfies SortTailwindClassesArgs, {
    name: "sortTailwindClasses",
  });
}
