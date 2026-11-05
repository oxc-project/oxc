// A suppressed statement keeps its source parens, so the leading `(` needs
// the `semi: false` ASI guard. Prettier misses it and its output re-parses
// as a call; the guarded line deviates
// (DIVERGENCES.md#suppressed-source-paren-asi-guard).

let x = 1;

// prettier-ignore
(sourceParen).sort();
