import { getRawTransferBuffer, parseRawSync, rawTransferSupported } from "../bindings.js";
import { buffers, registerBuffer } from "../plugins/lint.ts";
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
 * @throws {Error} If raw transfer is not supported on this platform, or parsing failed.
 *   A source whose text and AST exceed the 2 GiB buffer aborts the process instead.
 */
export function parse(path: string, sourceText: string, options?: ParseOptions): number {
  // Raw transfer is only supported on 64-bit little-endian systems
  if (!rawTransferSupported()) {
    throw new Error("`RuleTester` is not supported on 32-bit or big-endian systems");
  }

  const bufferId = parseRawSync(path, sourceText, options);

  // Get a view of the buffer from Rust the first time this module instance sees its ID.
  // A test runner that resets the module registry starts with an empty `buffers` and asks again.
  const buffer = buffers[bufferId] ?? registerBuffer(bufferId, getRawTransferBuffer());

  // Check parsing succeeded.
  // 0 is used as sentinel value to indicate parsing failed.
  // TODO: Get parsing error details from Rust to display nicely.
  const programOffset = buffer.int32[DATA_POINTER_POS_32];
  if (programOffset === 0) throw new Error("Parsing failed");

  return bufferId;
}
