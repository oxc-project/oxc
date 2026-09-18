// Issue #26742: the regexp absorbs statements from the initial script parse.
await /a(); b(); c(); d(); e(); f()/g;
// Preserve statements and comments after the reparsed region.
after();
export {};
