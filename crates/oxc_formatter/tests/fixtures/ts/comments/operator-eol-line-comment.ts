// An end-of-line line comment right after `=`/`:` keeps its position
// (`= // c` + mandatory break). Known divergence, see AGENTS.md:
// Prettier own-lines it for type aliases and union-valued property signatures,
// and flushes it past the member and its `;` for simple-typed ones
// (`simple: Value; // c`).
//
// `Value` below is NOT indented an extra level: annotation content after a `:`
// break gets no indent of its own — same family as variable/parameter/return
// type annotations, which are Prettier-identical in that shape. The union
// members ARE indented, by the union printer itself.

type Alias = // c
  "VALUE";

type AliasUnion = // c
  | AmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines
  | BmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines;

interface I {
  simple: // c
  Value;
  union: // c
    | AmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines
    | BmemberLongEnoughToMakeTheUnionTypeBreakIntoMultipleLines;
}
