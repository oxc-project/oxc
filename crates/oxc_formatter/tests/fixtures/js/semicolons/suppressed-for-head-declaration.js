// A suppressed `for` head declaration stays verbatim: it has no terminator of
// its own to re-add; Prettier appends one anyway and its output no longer parses.
// The last line deviates from Prettier (DIVERGENCES.md#suppressed-for-head-declaration).

for (/* prettier-ignore */ var i   =   1;;) [].sort()
