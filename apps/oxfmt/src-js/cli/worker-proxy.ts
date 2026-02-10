import Tinypool from "tinypool";
import { resolvePlugins } from "../libs/apis";
import type {
  FormatEmbeddedCodeParam,
  FormatFileParam,
  SortTailwindClassesArgs,
} from "../libs/apis";

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;

export async function initExternalFormatter(numThreads: number): Promise<string[]> {
  pool = new Tinypool({
    filename: new URL("./cli-worker.js", import.meta.url).href,
    minThreads: numThreads,
    maxThreads: numThreads,
    // XXX: Use `child_process` instead of `worker_threads`.
    // Not sure why, but when using `worker_threads`,
    // calls from NAPI (CLI) -> worker threads -> NAPI (prettier-plugin-oxfmt) causes a hang...
    runtime: "child_process",
  });

  return resolvePlugins();
}

export async function disposeExternalFormatter(): Promise<void> {
  await pool?.destroy();
  pool = null;
}

export async function formatEmbeddedCode(
  options: FormatEmbeddedCodeParam["options"],
  code: string,
): Promise<string> {
  return pool!.run({ options, code } satisfies FormatEmbeddedCodeParam, {
    name: "formatEmbeddedCode",
  });
}

export async function formatFile(
  options: FormatFileParam["options"],
  code: string,
): Promise<string> {
  return pool!.run({ options, code } satisfies FormatFileParam, {
    name: "formatFile",
  });
}

export async function sortTailwindClasses(
  options: SortTailwindClassesArgs["options"],
  classes: string[],
): Promise<string[]> {
  return pool!.run({ classes, options } satisfies SortTailwindClassesArgs, {
    name: "sortTailwindClasses",
  });
}
