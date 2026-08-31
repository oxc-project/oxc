// Flattening the output rope.
//
// `state.output` is a cons-string rope, one cell per `+=`. Whatever the caller does with the code
// (write it to a file, index into it, pass it to Rust) flattens it, and `generateSourceMap` scans it,
// which flattens it too. Doing it here, the way that is cheapest, is worth it.
//
// V8 has two different tree walkers.
//
// 1. `String::Flatten` (the C++ path a read takes - `charCodeAt`, regex, `Buffer`, NAPI) uses the newer `WriteToFlat2`.
// 2. The Torque string builtins use the classic `WriteToFlat`, which special-cases exactly the left-heavy list
//    that repeated `+=`s builds, and costs about half as much per cons cell for this pattern.
//
// `indexOf` (and `includes`) take the Torque path, and flatten the rope in place - the cons cell at the top
// becomes a wrapper around the new flat string, which every reader unwraps for free - and then scan for
// the search string, which stops at the first space.
//
// That is cheaper than `[s, " "].join("").slice(0, -1)`, the other route to the classic walker, which allocates
// an array and a slice on top of the copy, and hands back the slice rather than the flat string itself.
// A slice misses `JSON.stringify`'s fast path, among others.
//
// Output will contain a space close to the start in all but the weirdest cases, so the search itself is cheap.
//
// Other possible solutions which were tried and discarded:
// * `startsWith(" ")` / `endsWith(" ")` - with a constant search string TurboFan inlines them as `charCodeAt` loads,
//   whose non-flat path is `String::Flatten`, the slow walker.
// * `indexOf("")` / `includes("")` - they early return without flattening.
//
// A large output is not left to grow into one rope and flattened at the end. It is flattened in chunks along the way -
// see `OUTPUT_CHUNK_LENGTH` for why, and `printIndent` in `indent.ts` for where.

import type { State } from "../state.ts";

/**
 * Sink for the result of `indexOf`.
 *
 * TurboFan removes a call whose result is unused, and there would be no flatten - and nothing to notice it,
 * only the cost creeping back. The store to the object property is what keeps the call. Do not tidy it away.
 */
const SINK = { dummy: 0 };

/**
 * Flatten a rope in place.
 *
 * A no-op on a string which is already flat, apart from a short scan.
 */
export function flattenString(s: string): void {
  SINK.dummy ^= s.indexOf(" ");
}

/**
 * Length at which `state.output` is flattened and moved to `state.outputChunks`.
 *
 * Left to grow to the end of a large print, the rope costs in two separate ways, and flattening it
 * in chunks along the way removes both:
 *
 * 1. Any non-Latin1 character in the output (a string literal or JSX text with an accent, emoji, CJK,
 *    a typographic quote) makes the rope, and its flattened result, two-byte, and V8 copies a two-byte rope
 *    at ~12ns per cell against ~2ns for a one-byte one, on every flatten path. Each chunk is its own rope,
 *    so only the chunks which contain such a character pay that rate, not the whole output.
 *
 * 2. Everything the rope references is live. A young-generation GC landing mid-print copies every cell
 *    written so far, and promotes them at the next one. A print which allocates a fair share of the
 *    semi-space (which is 1MB in a fresh or idle process, and at most 32MB) sees one or more of those,
 *    and a big print sees them in almost every run. Flattened chunks are a few large objects instead.
 *
 * Measured against never flattening: two-byte outputs print 25-35% faster with any limit up to 32KB,
 * and one-byte outputs of 300KB+ print 5-25% faster once GC is counted. The cost is the per-line check
 * plus about half a microsecond per chunk: +1-4% on a 50-120KB one-byte print in a warm, roomy heap,
 * halving as the limit doubles. 16KB keeps almost all of the wins for half the cost of 8KB,
 * 64KB and above lose the two-byte win on files where such characters are dense, and lose to
 * never flattening in a 1MB semi-space, where the partial rope is itself most of what a GC copies.
 *
 * The check is in `printIndent`, which runs once per line at every nesting depth - so a chunk can overshoot
 * by a line, which doesn't really matter, the limit is not precise anyway. It's tempting to check less often,
 * e.g. once per top-level statement, but that wouldn't be often enough in files that have large top-level
 * statements, e.g. libraries written as one big IIFE.
 */
export const OUTPUT_CHUNK_LENGTH = 16 * 1024;

/**
 * Flatten `output` into `state.outputChunks` and start a new chunk.
 *
 * The chunks array is created on the first spill, so a small print never allocates it.
 */
export function spillOutputChunk(state: State): void {
  const { output } = state;
  state.output = "";

  // Only sourcemap builds need this
  if (SOURCEMAPS) state.spilledOutputLength += output.length;

  flattenString(output);
  const { outputChunks } = state;
  if (outputChunks === null) {
    state.outputChunks = [output];
  } else {
    outputChunks.push(output);
  }
}
