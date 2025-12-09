import { workerData } from "node:worker_threads";
import type { Options } from "prettier";

// Lazy load Prettier in each worker thread
//
// NOTE: In the past, statically importing caused issues with `oxfmt --lsp` not starting.
// However, this issue has not been observed recently, possibly due to changes in the bundling configuration.
// Nevertheless, we will keep it as lazy loading just in case.
let prettierCache: typeof import("prettier");

export type WorkerData = {
  prettierConfig: Options;
};

// Initialize config from `workerData` (passed during pool creation)
// NOTE: The 1st element is thread id, passed by `tinypool`
const [, { prettierConfig }] = workerData satisfies [unknown, WorkerData];

// ---

export type FormatEmbeddedCodeArgs = {
  parser: string;
  code: string;
};

export async function formatEmbeddedCode({
  parser,
  code,
}: FormatEmbeddedCodeArgs): Promise<string> {
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  return prettierCache
    .format(code, {
      ...prettierConfig,
      parser,
    })
    .then((formatted) => formatted.trimEnd())
    .catch(() => code);
}

// ---

export type FormatFileArgs = {
  parserName: string;
  fileName: string;
  code: string;
};

export async function formatFile({ parserName, fileName, code }: FormatFileArgs): Promise<string> {
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  return prettierCache.format(code, {
    ...prettierConfig,
    parser: parserName,
    filepath: fileName,
  });
}
