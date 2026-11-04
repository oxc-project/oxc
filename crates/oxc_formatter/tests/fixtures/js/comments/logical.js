// Kept isolated: a known NON-IDEMPOTENT shape (matches Prettier's first pass).
// The trailing `/* 2 */` rides the formatter-added precedence parens outward
// on the second pass; folding this into a pin fixture would pollute its
// idempotent snapshot.
code || !escapeless && (true /* 1 */ || false /* 2 */)
