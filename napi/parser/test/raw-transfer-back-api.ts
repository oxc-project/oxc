// Interface seam between the raw transfer back conformance suite and the implementation.
//
// This is the ONLY file the suite needs to touch when the implementation lands:
// * `RAW_TRANSFER_BACK_SUPPORTED` gates the whole suite (skipped while `false`).
// * `roundtrip()` performs: encode ESTree program into the arena buffer on the JS side
//   (`src-js/raw-transfer/serialize.js`), then call the native round-trip binding, which
//   placement-constructs the `Allocator` in the buffer, forms `&Program`, parses `sourceText`
//   independently, and returns the comparison results.
//
// Wiring TODO (PR 2 of raw_transfer_back, see `tasks/ast_tools/src/generators/raw_transfer_back.rs` plan):
// 1. Import `encodeRawBack` from `../src-js/raw-transfer/serialize.js`.
// 2. Import `rawTransferBackRoundtrip` binding from `../src-js/bindings.js`.
// 3. Set `RAW_TRANSFER_BACK_SUPPORTED = true` (or feature-detect the binding).

import type { Program } from "./parser.ts";

export const RAW_TRANSFER_BACK_SUPPORTED = false;

export interface RoundtripOptions {
  filename: string;
  // `null` = encode without source text: spans pass through unconverted and comments are
  // unavailable, so codegen comparison must run with comments disabled on both sides.
  sourceText: string | null;
  astType: "js" | "ts";
  preserveParens: boolean;
}

export interface RoundtripResult {
  // `ContentEq` between round-tripped and directly-parsed `Program` (spans excluded).
  contentEq: boolean;
  // Codegen output of the round-tripped `Program`.
  printed: string;
  // Codegen output of the directly-parsed `Program`.
  expected: string;
}

// oxlint-disable-next-line no-unused-vars
export function roundtrip(program: Program, options: RoundtripOptions): RoundtripResult {
  throw new Error(
    "raw_transfer_back is not implemented yet. " +
      "Wire this function to `encodeRawBack` + the `rawTransferBackRoundtrip` binding, " +
      "and set `RAW_TRANSFER_BACK_SUPPORTED = true`.",
  );
}
