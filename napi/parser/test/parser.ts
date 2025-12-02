/*
 * Re-export `parse` and `parseSync` functions with type definitions patched
 * to expose hidden `experimentalRawTransfer`, `experimentalLazy`, and `experimentalParent` options.
 */

import * as parser from "../src-js/index.js";

import type { ParserOptions as OriginalParserOptions } from "../src-js/index.js";

export type * from "../src-js/index.js";

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

export const parse: OverrideOptions<typeof parser.parse> = parser.parse;
export const parseSync: OverrideOptions<typeof parser.parseSync> = parser.parseSync;
