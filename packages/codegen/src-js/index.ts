// Entry point. Dispatches to whichever build of the printer suits the options it's given.
//
// `DESIGN.md` in the package root is the overview of the whole printer - why a JS printer beats
// a native one here, where this port diverges from Rust `oxc_codegen` (notably `state.last`),
// and what makes it fast. Read it before changing anything below, or in `print`.
//
// The printer is built 4 times from `print/`, over 2 build-time feature flags.
//
// 1. Source map support - costs a little speed even when it's unused.
// 2. TypeScript support - costs JS-only ASTs a swathe of field checks and switch arms.
//
// The most important effect of separate JS and TS builds is not the reduced logic when printing a JS AST,
// but that they receive differently-shaped AST node objects. If both JS-shape and TS-shape ASTs
// pass through the same functions, every function becomes polymorphic, which is a large slow-down.
// Having specialized code paths for JS and TS-shape ASTs avoids this problem - most functions remain monomorphic.
//
// Each combination is its own build, lazy-loaded depending on the `sourcemap` and `ts` options.
//
// `require` rather than `import()` because the printer is synchronous.
// All builds are ESM, which `require` can load on the Node versions in this package's `engines` field.

import { createRequire } from "node:module";
import { State } from "./state.ts";

import type { CodegenResult, Options } from "./print/options.ts";
import type * as ESTree from "../../../npm/oxc-types/types.d.ts";

export type { CodegenResult, Options, SourceMap } from "./print/options.ts";

/**
 * All printer builds are compiled from `print/index.ts`, so they share its exports.
 */
type PrintModule = typeof import("./print/index.ts");

const require = createRequire(import.meta.url);

/**
 * Passed on when the caller supplies no options, so the printer never has to check for their absence.
 */
const EMPTY_OPTIONS: Options = {};

/**
 * Printer builds, indexed by `(ts ? 1 : 0) + (sourcemap ? 2 : 0)`. Lazily loaded as needed.
 */
const PRINTER_PATHS = [
  "./print_js.js",
  "./print_ts.js",
  "./print_js_maps.js",
  "./print_ts_maps.js",
];

/**
 * The loaded builds, in the same order as `PRINTER_PATHS`, and `null` until first used.
 */
const printers: (PrintModule["printSync"] | null)[] = [null, null, null, null];

/**
 * Print `node`, returning an object including the generated code.
 *
 * @param node - AST node to print, a `Program` or a single statement
 * @param options - Printing options (optional)
 * @returns Object holding the generated code
 */
export function printSync(
  node: ESTree.Program | ESTree.Statement,
  options?: Options,
): CodegenResult {
  // The printer is built 4 times, over whether the AST may contain TypeScript and whether
  // source mappings are wanted. This picks the build the options call for and loads it on first use,
  // so a caller printing only JavaScript never pays for the TypeScript printers.
  let index = 0;
  if (options == null) {
    options = EMPTY_OPTIONS;
  } else {
    if (options.ts === true) index = 1;
    if (options.sourcemap === true) {
      if (typeof options.sourceText !== "string") {
        throw new TypeError("`sourceText` must be a string when `sourcemap` is true");
      }
      if (options.sourceFilename !== undefined && typeof options.sourceFilename !== "string") {
        throw new TypeError("`sourceFilename` must be a string when supplied");
      }
      index |= 2;
    }
  }

  let print = printers[index];
  if (print === null) {
    print = (require(PRINTER_PATHS[index]) as PrintModule).printSync;
    printers[index] = print;
  }

  // State is created here, not in the printer, so that all 4 builds share one class
  // and therefore see one object shape
  return print(node, new State(options), options);
}
