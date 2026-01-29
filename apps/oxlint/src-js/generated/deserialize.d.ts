// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

import type { Program } from "./types.d.ts";
import type { Location as SourceLocation } from "../plugins/location.ts";

type BufferWithArrays = Uint8Array & { uint32: Uint32Array; float64: Float64Array };
type GetLoc = (node: { range: [number, number] }) => SourceLocation;

export declare function deserializeProgramOnly(
  buffer: BufferWithArrays,
  sourceText: string,
  sourceStartPosInput: number,
  sourceByteLen: number,
  getLoc: GetLoc,
): Program;

export declare function resetBuffer(): void;
