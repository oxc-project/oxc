// A single-member union loses its parens along with the dropped `|`.
// Issue #18941: also in array-element position
type Items = ( | number)[];
type Items2 = ( & number)[];
// Multi-member unions keep the parens
type Items3 = (string | number)[];
type Simple = | number;
// The paren payload itself can be any type
type T1<B> = | (B extends any ? number : string);
type T2 = | (() => void);
