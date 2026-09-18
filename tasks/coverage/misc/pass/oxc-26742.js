// Reparsing the division chain as an awaited regexp must consume every old statement.
await /a(); b(); c(); d(); e(); f()/g;
export {};
