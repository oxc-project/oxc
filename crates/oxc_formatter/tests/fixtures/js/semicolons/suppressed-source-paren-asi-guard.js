// A suppressed statement keeps its source parens, so the leading `(` needs
// the `semi: false` ASI guard.

let x = 1;

// prettier-ignore
(sourceParen).sort();
