// A `for of` head cannot start with `let`, so these must stay parenthesized
for ((let).x of y);
for ((let.a).x of y);
for ((let).a[0] of y);

// `let[...]` prints as `(let)[...]`, which no longer starts with `let`, so these need no more parens
for ((let)[0] of y);
for ((let)[0].x of y);
