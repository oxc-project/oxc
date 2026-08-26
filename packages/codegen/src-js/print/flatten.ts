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
