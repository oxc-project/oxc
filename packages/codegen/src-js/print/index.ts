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
import { emitMappings } from "./source_map.ts";
import { printProgram, printStatement } from "./statement.ts";

import type { Options } from "./options.ts";
import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Print `node`, returning the generated code.
 *
 * One of these exists per build. The package entry point picks the build and calls it -
 * callers of the package never reach this directly.
 *
 * @param state - Created by the entry point, so that all 4 builds share one class and so one object shape
 * @param options - The same options `state` was created from, for the parts only this build acts on
 * @returns Generated code
 */
export function printSync(node: ESTree.Node, state: State, options: Options): string {
  if (node.type === "Program") {
    printProgram(node, state);
  } else {
    typeAssertIs<ESTree.Statement>(node);
    printStatement(node, state);
  }

  // This is removed by minifier in non-sourcemap builds
  if (SOURCEMAPS) {
    debugAssert(
      options.sourceMap != null,
      "`options.sourceMap` should be defined when `SOURCEMAPS` is `true`",
    );

    emitMappings(state, options.sourceMap);
  }

  return state.output;
}

export type { Mapping, Options, Position, SourceMapGenerator } from "./options.ts";
