// A cast target keeps its parens, so a same-operator chain is not flattened through it.
x = a && /** @type {T} */ (b && c);
x = /** @type {T} */ (b && c) && a;
x = a && (/** @type {T} */ (b && c) && d);
x = /** @type {T} */ (b + c) + a;

// Same for an arrow chain.
f = (a) => /** @type {T} */ ((b) => c);

// A plain comment is not a cast, the chain flattens.
x = a && /* c */ (b && c);
