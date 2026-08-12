// Indentation.
//
// One level's worth of indentation is a string, and the strings for each level are cached,
// since the same handful of levels are printed over and over.
//
// The cache is not this build's. It is created once per process alongside `State`, which carries it here,
// so the levels one build has grown are there for the next one - see `state.ts`.

import { CAT_OTHER, write } from "./write.ts";

import type { State } from "../state.ts";

/**
 * Write the indentation for the current level, or the pending single space in place of it.
 *
 * Inline statement bodies ask for a space where an indent would go, which keeps `if (x) foo()` on one line.
 */
export function printIndent(state: State): void {
  if (state.pendingIndentAsSpace) {
    write(state, " ", CAT_OTHER);
    state.pendingIndentAsSpace = false;
    return;
  }

  const level = state.indentLevel;
  if (level > 0) {
    // Indentation is validated to contain only spaces and tabs, neither of which any reader
    // of `last` distinguishes, so the category is a constant
    const { indents } = state;
    write(state, level < indents.length ? indents[level] : growIndents(state, level), CAT_OTHER);
  }
}

/**
 * Extend the indent cache up to `level` and return the indent for it.
 *
 * Separate from `printIndent` so the cached lookup there stays small enough to inline.
 */
function growIndents(state: State, level: number): string {
  const { indents, indentString } = state;

  let { length } = indents;
  let indent = indents[length - 1];
  for (; length <= level; length++) {
    indent += indentString;
    // Force the cons string flat.
    // That costs here, but it is appended to the output many times afterwards.
    indent.charCodeAt(0);
    indents.push(indent);
  }

  return indent;
}
