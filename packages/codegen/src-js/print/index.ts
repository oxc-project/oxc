// A fast JavaScript code generator from an ESTree-compliant AST.
//
// A faithful port of Oxc's Rust `oxc_codegen` crate (pretty-printing mode).
// Output is byte-identical to `oxc_codegen` with default options (tab indentation, double quotes, comments off).
//
// The printer is one function per construct, split across this directory by area.
// `printStatement` in `statement.ts` and `printExpression` in `expression.ts` are the two dispatch
// points the rest hangs off.
//
// Reference: `oxc/crates/oxc_codegen/src/{gen.rs,lib.rs,str.rs,binary_expr_visitor.rs}`

import { debugAssert, typeAssertIs } from "../asserts.ts";
import { flattenString } from "./flatten.ts";
import { generateSourceMap } from "./source_map.ts";
import { printProgram, printStatement } from "./statement.ts";

import type { CodegenResult, Options } from "./options.ts";
import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print `node`, returning an object including the generated code.
 *
 * One of these exists per build. The package entry point picks the build and calls it -
 * callers of the package never reach this directly.
 *
 * The result is an object so that further outputs (a source map) can be added without a breaking change.
 *
 * @param state - Created by the entry point, so that all 4 builds share one class and so one object shape
 * @param options - The same options `state` was created from, for the parts only this build acts on
 * @returns Object holding the generated code
 */
export function printSync(node: ESTree.Node, state: State, options: Options): CodegenResult {
  if (node.type === "Program") {
    printProgram(node, state);
  } else {
    typeAssertIs<ESTree.Statement>(node);
    printStatement(node, state);
  }

  // Flatten the output before handing it on - see `flatten.ts` for why, and why this way.
  //
  // A large print has already flattened most of its output into `state.outputChunks` chunk by chunk (see `flatten.ts`),
  // leaving `state.output` as the tail. Those flat chunks and the tail are joined here, which is a straight copy,
  // and the result is a proper flat string.
  const { outputChunks } = state;
  let output;
  if (outputChunks === null) {
    output = state.output;
    flattenString(output);
  } else {
    // `state.output` may be empty, which `join` ignores. A lone chunk comes back as-is, and it is already flat.
    outputChunks.push(state.output);
    output = outputChunks.join("");
    // The chunks are now redundant. Drop them now rather than when `state` dies, so a GC during `generateSourceMap`
    // collects them, instead of pointlessly retaining and copying them
    state.outputChunks = null;
  }

  // This is removed by minifier in non-sourcemap builds.
  //
  // The `skipSourcemapGeneration` check exists only in benchmark builds -
  // everywhere else `BENCHMARKS` is `false` and the condition folds back to `SOURCEMAPS` alone.
  // The option is deliberately not in `Options` - types are documentation, and it is not public API.
  // @ts-expect-error `skipSourcemapGeneration` is benchmarks-only, so is not in `Options`
  if (SOURCEMAPS && (!BENCHMARKS || !options.skipSourcemapGeneration)) {
    debugAssert(options.sourcemap === true, "`options.sourcemap` should be true in a maps build");
    return { code: output, map: generateSourceMap(state, output, options) };
  }

  return { code: output, map: null };
}
