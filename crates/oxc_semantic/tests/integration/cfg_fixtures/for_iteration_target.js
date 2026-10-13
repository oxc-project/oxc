// Branching targets make their evaluation visible in the CFG snapshot.
for ((getTarget() || fallback()).x of getValues()) {
    body();
    continue;
}
afterOf();

for (getTarget()[getKey() || defaultKey()] in getObject()) {
    body();
}
afterIn();

// Legacy initializers still run once, before collection evaluation.
for (var key = initialize() || fallback() in getObject()) {
    body(key);
    break;
}
afterInitializer();
