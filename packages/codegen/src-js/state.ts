// Printer state.
//
// One object, created here and handed to whichever printer build `index.ts` selects.
// It is deliberately outside `print/`, so that all 4 builds share this one class and therefore one
// object shape - a build-time flag must never add or remove a field, or the printers would each
// see a different hidden class for the object they thread through every function.
//
// The printers only ever receive it, so they import it as a type.
// Nothing in `print/` constructs one, and none of the 4 printer builds bundles this file.

import { CAT_OTHER } from "./print/categories.ts";
import { debugAssert } from "./asserts.ts";

import type { Category } from "./print/categories.ts";
import type { Options } from "./print/options.ts";

/**
 * The string one level of indentation prints as. Set from the `indent` option, a tab by default.
 */
let indentString = "\t";

/**
 * Indentation cache, holding the string for each level - `indents[2]` is two levels' worth.
 *
 * There is one per process, and every `State` carries it, so all 4 printer builds grow and read
 * the same array - the cache a build fills is there for the next build to use.
 *
 * It survives across prints and is discarded only when `indentString` changes.
 */
const indents = [""];

/** The `indent` option must be a non-empty string made up of only spaces and tabs. */
const INDENT_REGEX = /^[ \t]+$/;

/** Upper bound for the process-wide indentation cache. */
const MAX_STARTING_INDENT_LEVEL = 1_000;

/**
 * Initial capacity of the process-wide mapping-positions buffer.
 * Must be even - positions are recorded in pairs, and the buffer only grows between pairs.
 */
const MIN_MAP_POSITIONS_LENGTH = 1024;

debugAssert(
  MIN_MAP_POSITIONS_LENGTH % 2 === 0,
  "`MIN_MAP_POSITIONS_LENGTH` must be an even number",
);

/**
 * Deferred source mapping positions - pairs of output offset and source offset.
 *
 * Like `indents`, there is one buffer per process, and every maps-enabled `State` carries it,
 * so both sourcemap builds record into the same buffer.
 *
 * `growMapPositions` doubles it when a print fills it.
 */
let mapPositions = new Int32Array(MIN_MAP_POSITIONS_LENGTH);

export class State {
  // Current output.
  // A string which is appended to as the printing process proceeds.
  declare output: string;

  // Flattened chunks of `output`, in order, once it has grown past `OUTPUT_CHUNK_LENGTH` (see `print/flatten.ts`),
  // or `null` while it has not - a small print never allocates the array. Sourcemap builds also keep the total
  // length spilled so far - the true output offset the recorders need is `spilledOutputLength + output.length`.
  declare outputChunks: string[] | null;
  declare spilledOutputLength: number;

  // Current indentation level.
  declare indentLevel: number;

  // The process-wide indent cache and the string one level of it prints as.
  // `printIndent` reads the cache through here, and `growIndents` extends it.
  declare indents: string[];
  declare indentString: string;

  // `true` if printing a JSX / TSX file. Only affects printing of TSX.
  declare isJsx: boolean;

  // Operator/token glue tracking.
  declare pendingIndentAsSpace: boolean;

  // Category of the last thing written - one of the `CAT_*` constants, never a character.
  //
  // It also carries the three `CAT_START_OF_*` markers, which say that an expression statement,
  // a concise arrow body or an `export default` expression is about to be printed, and nothing
  // has been written since.
  //
  // A node seeing one of those is the leftmost token of that construct, however deeply nested it is,
  // which is how an object literal, a function or a class knows to parenthesize itself.
  //
  // They work because the next write replaces `last`, so "nothing written since" needs nothing else recorded -
  // and because every position they mark follows whitespace, which is what the `CAT_OTHER` they displace
  // means for spacing.
  declare last: Category;

  // `true` between a `writeNoLast` and the write which follows it,
  // i.e. while `last` describes something other than what was written last.
  // Only used in debug builds. See `debugAssertLastFresh`.
  declare lastIsStale: boolean;

  // The character the output currently ends with.
  // Only used in debug builds. See `debugAssertCategoryMatches`.
  declare lastCharWritten: string;

  // Deferred source mappings. When source maps are enabled, `mapPositions` is the process-wide
  // buffer above, holding generated/source offset pairs, and `mapPositionsLen` is how much of it
  // this print has filled.
  // Names are sparse, so their index/name pairs exist only if a mapping carries an original name.
  declare mapPositions: Int32Array | null;
  declare mapPositionsLen: number;
  declare mapNames: (number | string)[] | null;

  // Original source text, used to preserve names in source maps when the caller provides it.
  declare sourceText: string | null;

  constructor(options: Options) {
    this.output = "";
    this.outputChunks = null;
    this.spilledOutputLength = 0;

    let { startingIndentLevel: indentLevel } = options;
    if (indentLevel === undefined) {
      indentLevel = 0;
    } else if (
      !Number.isSafeInteger(indentLevel)
      || indentLevel < 0
      || indentLevel > MAX_STARTING_INDENT_LEVEL
    ) {
      throw new RangeError(
        "`startingIndentLevel` must be a non-negative safe integer no greater than 1000",
      );
    }
    this.indentLevel = indentLevel;

    // The `indent` option is validated here, not in the printer, and changing of it discards
    // the cache grown for the old `indentString`.
    // That should be rare - most users have an indent style they prefer, and use it consistently.
    const { indent: indentOption } = options;
    let indent = indentOption;
    if (indent === undefined) {
      indent = "\t";
    } else if (typeof indent !== "string" || !INDENT_REGEX.test(indent)) {
      throw new TypeError("`indent` must be a non-empty string containing only spaces and tabs");
    }
    if (indent !== indentString) {
      indentString = indent;
      indents.length = 1;
    }
    this.indents = indents;
    this.indentString = indent;

    // `.tsx` mode: lone type parameters print as `<T,>`
    this.isJsx = options.jsx === true;

    // Operator/token glue tracking
    this.pendingIndentAsSpace = false;

    // Start of output behaves like after whitespace. Reading the last character from `output`
    // directly would flatten V8's rope representation on every append-then-read (quadratic),
    // which is why the category is tracked rather than derived.
    this.last = CAT_OTHER;

    // Debug-only fields for checking `last` is correct on both writes and reads
    if (DEBUG) {
      this.lastIsStale = false;
      this.lastCharWritten = "";
    }

    // `writeWithMap*` functions record the output offset and original position of every mapped node,
    // and `generateSourceMap` encodes them in one pass at the end
    if (options.sourcemap !== true) {
      this.sourceText = null;
      this.mapPositions = null;
      this.mapNames = null;
    } else {
      debugAssert(options.sourceText != null);
      this.sourceText = options.sourceText;
      this.mapPositions = mapPositions;
      this.mapNames = [];
    }
    this.mapPositionsLen = 0;
  }

  /**
   * Replace the full mapping-positions buffer with one twice the size, copying its contents over.
   *
   * The new buffer also replaces the process-wide one, so later prints start at the new size.
   *
   * @returns The new buffer
   */
  growMapPositions(): Int32Array {
    const oldBuffer = this.mapPositions;
    debugAssert(
      oldBuffer === mapPositions,
      "`growMapPositions` should only be called on a `State` with source maps enabled",
    );
    const newBuffer = new Int32Array(oldBuffer.length * 2);
    newBuffer.set(oldBuffer);
    mapPositions = newBuffer;
    this.mapPositions = newBuffer;
    return newBuffer;
  }
}
