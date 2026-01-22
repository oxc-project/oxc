// Async arrow function without await, should trigger lint
// error if type-aware rules are enabled.
[1, 2, 3].map(async (x) => x + 1);
