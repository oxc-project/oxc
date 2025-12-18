import type { Options } from "prettier";

// Lazy load Prettier in each worker thread
//
// NOTE: In the past, statically importing caused issues with `oxfmt --lsp` not starting.
// However, this issue has not been observed recently, possibly due to changes in the bundling configuration.
// Nevertheless, we will keep it as lazy loading just in case.
let prettierCache: typeof import("prettier");

// ---

export type FormatEmbeddedCodeArgs = {
  code: string;
  options: Options;
};

export async function formatEmbeddedCode({
  options,
  code,
}: FormatEmbeddedCodeArgs): Promise<string> {
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  return prettierCache
    .format(code, options)
    .then((formatted) => formatted.trimEnd())
    .catch(() => code);
}

// ---

export type FormatFileArgs = {
  code: string;
  options: Options;
};

export async function formatFile({ options, code }: FormatFileArgs): Promise<string> {
  if (!prettierCache) {
    prettierCache = await import("prettier");
  }

  return prettierCache.format(code, options);
}
