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
    // TODO remove this, enabling for test
    embeddedLanguageFormatting: "auto",
    // TODO: cache this
    plugins: await getPlugins(prettierCache),
  });
}

async function getPlugins(prettier) {
  const { languages } = await prettier.getSupportInfo();

  const programParser = {
    astFormat: "oxfmt",
    parse: (text) => ({
      type: "Program",
      body: [],
      sourceType: "module",
      __oxfmtFormatted: "formatted by oxfmt",
    }),
  };

  const expressionParser = {
    astFormat: "oxfmt",
    parse: (text) => ({
      type: "Program",
      body: [],
      sourceType: "module",
      __oxfmtFormatted: "formatted by oxfmt expr",
    }),
  };

  return [
    {
      // TODO: double check whether we need this
      languages: languages
        .filter((l) => ["JavaScript", "JSX", "TypeScript", "TSX"].includes(l.name))
        .map((l) => ({ ...l, parsers: ["oxc"] })),

      parsers: {
        oxc: programParser,
        babel: programParser,
        "babel-ts": programParser,
        typescript: programParser,
        __vue_expression: expressionParser,
        __vue_ts_expression: expressionParser,
        __vue_event_binding: expressionParser,
        __vue_ts_event_binding: expressionParser,
        __js_expression: expressionParser,
        __ts_expression: expressionParser,
      },

      printers: {
        oxfmt: {
          print: (path) => path.node.__oxfmtFormatted || "/* ERROR */",
        },
      },
    },
  ];
}
