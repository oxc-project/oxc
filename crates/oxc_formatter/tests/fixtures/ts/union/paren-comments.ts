// Comments around a union's formatter-added parentheses:
// inline comments keep their source side of the `(`, own-line comments stay above it.

type AO = /* c */ (A | B)[];
type AI = (/* c */ A | B)[];
// NOTE: Known divergence — Prettier prints `keyof (/* c */ A | B)`, normalizing only
// this context to inside while keeping array/indexed-access outside; we keep the
// source side everywhere (see AGENTS.md "Known divergences").
type KO = keyof /* c */ (A | B);
type KI = keyof (/* c */ A | B);
type IO = /* c */ (A | B)["k"];
type II = (/* c */ A | B)["k"];
type OO = [/* c */ (A | B)?];
type OI = [(/* c */ A | B)?];
type XO = X & /* c */ (A | B);
type XI = X & (/* c */ A | B);
type UO = U | /* c */ (A | B);
type UI = U | (/* c */ A | B);

// Own-line comments hoist out of the added paren (matches Prettier, prettier#18379)
type OL = (
  A | B // comment 1
) & (
  // comment2
  A | B
);

// Known limitation of the hoist (matches Prettier): when the union expands, the added
// `(` takes its own line between a next-line directive and its former target line,
// detaching the directive (here: the first member's line is no longer covered, and
// `@ts-expect-error` turns into an "Unused directive" error). Collapsed unions are safe.
type DirectiveWindow = X & (
  // @ts-expect-error
  AaaaaaaaaaaaaaaaaaaaaaaaaaaaMember | BbbbbbbbbbbbbbbbbbbbbbbbbbbbMember | CcccccccccccccccccccccccccccMember
);
