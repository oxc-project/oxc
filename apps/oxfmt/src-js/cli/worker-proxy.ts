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
    // Use child_process instead of worker_threads to avoid issues
    // when loading NAPI modules within Prettier plugin
    runtime: "child_process",
  });

  return resolvePlugins();
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
