// The `semi: false` ASI guard prints BEFORE a leading type cast comment even
// for a suppressed statement: a `;` between the cast comment and its parens
// detaches the cast (tsc drops the type). Prettier prints it after the comment;
// the guarded line deviates (DIVERGENCES.md#suppressed-cast-comment-asi-guard).

// prettier-ignore
/** @type {string[]} */ (cast).sort();
