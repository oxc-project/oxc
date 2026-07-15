// ESTree -> arena encoder seam for raw transfer back.
//
// THE SINGLE WIRING POINT for the `raw_transfer_back` implementation. When the encoder lands:
//
// 1. The `RawTransferBackGenerator` in `tasks/ast_tools` emits the generated encoder into
//    `src-js/generated/serialize/{js,ts}.js` (one serializer function per AST type, writing
//    real oxc struct layouts into the aligned buffer, emulating the downward bump allocator).
// 2. `encodeProgram` below acquires a pooled buffer via `./buffer.js`, writes the source text
//    (when provided), reserves the in-buffer `Allocator` slot, runs the generated
//    `serializeProgram`, writes the metadata, and returns
//    `{ buffer, programOffset, sourceStart, sourceLen }`.
// 3. The conformance suite in `napi/parser/test/raw-transfer-back-api.ts` wires to this same
//    encoder (via workspace devDependency), flipping `RAW_TRANSFER_BACK_SUPPORTED` there.

/**
 * Encode an ESTree `Program` into an aligned buffer as oxc arena structs.
 *
 * @param {Object} program - ESTree `Program` object (oxc-parser shaped)
 * @param {string|null|undefined} sourceText - Original source text, or `null` / `undefined`
 * @returns {Object} - Object of form `{ buffer, programOffset, sourceStart, sourceLen }`
 * @throws {Error} - Always, until the `raw_transfer_back` encoder is implemented
 */
// oxlint-disable-next-line no-unused-vars
export function encodeProgram(program, sourceText) {
  throw new Error(
    "raw_transfer_back encoder is not implemented yet. " +
      "`print` will work once the ESTree -> arena serializer lands in `src-js/generated/serialize/`.",
  );
}
