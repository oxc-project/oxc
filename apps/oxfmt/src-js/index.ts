import * as napi from "./bindings.js";
import { setupConfig, formatEmbeddedCode, formatFile } from "./prettier-proxy.js";
import type { Oxfmtrc } from "./bindings.js";

// NOTE: We need to re-export related types and enum fields manually
export type { Oxfmtrc } from "./bindings.js";
export {
  EndOfLineConfig,
  QuotePropsConfig,
  ArrowParensConfig,
  EmbeddedLanguageFormattingConfig,
  ObjectWrapConfig,
  TrailingCommaConfig,
  SortOrderConfig,
} from "./bindings.js";

export async function format(fileName: string, sourceText: string, options?: Oxfmtrc) {
  if (typeof fileName !== "string") throw new TypeError("`fileName` must be a string");
  if (typeof sourceText !== "string") throw new TypeError("`sourceText` must be a string");

  return napi.format(
    fileName,
    sourceText,
    options ?? {},
    setupConfig,
    formatEmbeddedCode,
    formatFile,
  );
}
