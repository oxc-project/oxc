import { parseRawSync, rawTransferSupported } from "../bindings.js";
import { registerBuffer } from "../plugins/lint.ts";
import { DATA_POINTER_POS_32 } from "../generated/constants.ts";

import type { ParserOptions as ParseOptions } from "../bindings.js";

export type { ParseOptions };

/**
 * Parse source text into a buffer, ready for `lintFileImpl`.
 *
 * Rust owns the buffer and shares it with JS, the same as when the linter runs JS plugins.
 * The buffer's contents remain valid until the next call to `parse`.
 *
 * @param path - Path of file to parse
 * @param sourceText - Source text to parse
 * @param options - Parsing options
 * @returns ID of the buffer the AST was written into
 * @throws {Error} If raw transfer is not supported on this platform, or parsing failed
 */
export function parse(path: string, sourceText: string, options?: ParseOptions): number {
  // Raw transfer is only supported on 64-bit little-endian systems
  if (!rawTransferSupported()) {
    throw new Error("`RuleTester` is not supported on 32-bit or big-endian systems");
  }

  // `buffer` is `undefined` if Rust already sent the buffer with this ID to JS
  const { bufferId, buffer: newBuffer } = parseRawSync(path, sourceText, options);
  const buffer = registerBuffer(bufferId, newBuffer ?? null);

  // Check parsing succeeded.
  // 0 is used as sentinel value to indicate parsing failed.
  // TODO: Get parsing error details from Rust to display nicely.
  const programOffset = buffer.int32[DATA_POINTER_POS_32];
  if (programOffset === 0) throw new Error("Parsing failed");

  return bufferId;
}
