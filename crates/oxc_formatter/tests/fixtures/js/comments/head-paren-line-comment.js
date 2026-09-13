// A line comment inside a statement head's parentheses stays inside them,
// flushed before the `)`.
// - for-in / for-of: diverges — Prettier is not idempotent (prettier#12880),
//   its first pass moves the comment past the body's `{`
// - with / while: no divergence (their grouped heads break), the control cases
for (a in b // c
) {}

for (a of b // c
) {}

with (a // c
) {}

while (x // c
) {}

// The remaining paren-headed constructs need no special handling
// (their grouped heads + the generic trailing pass already flush before `)`,
// also across a parenthesized inner expression's re-printed `)`):
switch ((0, 0) // c
) { case a: b(); }

do {} while ((0, 0) // c
);

for (a; b; (0, 0) // c
) {}

try {} catch (e // c
) {}

// Also with an empty-statement body; for-in diverges here too
// (Prettier moves the comment past the `)` and the `;`: `for (a in b); // c`)
for (a in b // c
);

if (x // c
);
