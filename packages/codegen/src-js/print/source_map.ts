// Source map generation.
//
// `write` records the output offset and original position of every mapped node as it goes,
// and this converts those offsets to generated line/column in one pass at the end.

import { debugAssert, typeAssertIs } from "../asserts.ts";

import type { Mapping, MutableMapping, SourceMapGenerator } from "./options.ts";
import type { State } from "../state.ts";

/**
 * Convert the offsets recorded during printing into mappings, and hand them to the generator.
 *
 * Printing records only an output offset per mapped node, which is why this walks the output once
 * at the end counting newlines, rather than tracking a line and column throughout.
 *
 * @param sourceMap - Generator the mappings are added to, which the caller was given in its options
 */
export function emitMappings(state: State, sourceMap: SourceMapGenerator): void {
  debugAssert(
    state.mapOffsets !== null && state.mapPositions !== null && state.mapNames !== null,
    "Source map arrays should exist when a `sourceMap` was given",
  );

  const { output, mapOffsets, mapPositions, mapNames } = state;
  const source = sourceMap.file || sourceMap._file;

  // The `generated` and `mapping` objects are reused across `addMapping` calls to avoid generating ephemeral objects
  const generated = { line: 1, column: 0 };
  const mapping: MutableMapping = { original: null, generated, name: undefined, source };

  let line = 1;
  let lineStart = 0;
  let nextNewline = output.indexOf("\n");
  if (nextNewline === -1) nextNewline = Infinity;

  const { length } = mapOffsets;
  for (let i = 0; i < length; i++) {
    const offset = mapOffsets[i];
    while (offset > nextNewline) {
      line++;
      lineStart = nextNewline + 1;
      const next = output.indexOf("\n", lineStart);
      nextNewline = next === -1 ? Infinity : next;
    }

    generated.line = line;
    generated.column = offset - lineStart;
    mapping.original = mapPositions[i];
    mapping.name = mapNames[i];

    typeAssertIs<Mapping>(mapping);
    sourceMap.addMapping(mapping);
  }
}
