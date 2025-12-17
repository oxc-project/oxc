/*
 * Re-export `parse` and `parseSync` functions with type definitions patched
 * to expose hidden `experimentalRawTransfer`, `experimentalLazy`, and `experimentalParent` options.
 */

import * as parser from "#oxc-parser";

import type { ParserOptions as OriginalParserOptions } from "#oxc-parser";

export type * from "#oxc-parser";

interface ExperimentalParserOptions {
  experimentalRawTransfer?: boolean;
  experimentalParent?: boolean;
  experimentalLazy?: boolean;
}

export type ParserOptions = OriginalParserOptions & ExperimentalParserOptions;

type OverrideOptions<F extends (...args: any[]) => any> = F extends (
  filename: infer A,
  sourceText: infer B,
  options?: infer C,
) => infer R
  ? (filename: A, sourceText: B, options?: C & ExperimentalParserOptions) => R
  : never;

interface ParseMethods {
  parse: OverrideOptions<typeof parser.parse>;
  parseSync: OverrideOptions<typeof parser.parseSync>;
}

export const { parse, parseSync }: ParseMethods = parser;
