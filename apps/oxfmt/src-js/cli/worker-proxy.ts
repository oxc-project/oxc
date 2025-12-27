import Tinypool from "tinypool";
import { resolvePlugins } from "../libs/prettier";
import type { FormatEmbeddedCodeParam, FormatFileParam } from "../libs/prettier";
import type { Options } from "prettier";

// Worker pool for parallel Prettier formatting
let pool: Tinypool | null = null;

export async function initExternalFormatter(numThreads: number): Promise<string[]> {
  pool = new Tinypool({
    filename: new URL("./cli-worker.js", import.meta.url).href,
    minThreads: numThreads,
    maxThreads: numThreads,
  });

  return resolvePlugins();
}

export async function formatEmbeddedCode(
  options: Options,
  tagName: string,
  code: string,
): Promise<string> {
  return pool!.run({ options: toPlainOptions(options), code, tagName } satisfies FormatEmbeddedCodeParam, {
    name: "formatEmbeddedCode",
  });
}

export async function formatFile(
  options: Options,
  parserName: string,
  fileName: string,
  code: string,
): Promise<string> {
  return pool!.run({ options: toPlainOptions(options), code, fileName, parserName } satisfies FormatFileParam, {
    name: "formatFile",
  });
}

function toPlainOptions(options: Options) {
  // now bun napi value could not cloned
  // but for Web Workers in Bun, structuredClone used.
  // https://github.com/oven-sh/bun/issues/25658
  if ((globalThis as any).Bun) {
    return {...options}
  }
  return options
}
