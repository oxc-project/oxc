// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/raw_transfer.rs`.

import type * as ESTree from "@oxc-project/types";

type BufferWithArrays = Uint8Array & { uint32: Uint32Array; float64: Float64Array };

export declare function deserialize(
  buffer: BufferWithArrays,
  sourceText: string,
  sourceByteLen: number,
): ESTree.Program;

export declare function resetBuffer(): void;
