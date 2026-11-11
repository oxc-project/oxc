// A single-line block comment before a union's dropped `(` is the first member's from the first pass.
// DIVERGES from Prettier (DIVERGENCES.md#union-hug-comment-before-dropped-paren), which hugs first and expands on its second pass.
type Alias = /* c */ ({ aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccccccccccc: 1 } | null);
function param(a: /* c */ ({ aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; cccccccccccccccc: 1 } | null)) {}

// Kept parentheses keep the comment outside, and the hug
type Keyof = keyof /* c */ ({ aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; ccccccccccccccccccc: 1 } | null);
type ArrayOf = /* c */ ({ aaaaaaaaaaaaaaaaaaaa: string; bbbbbbbbbbbbbbbbbbbbbbbb: number; ccccccccccccccccccccc: 1 } | null)[];
