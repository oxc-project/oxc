class C {
    get accessor() { return 1; }
}
babelHelpers.decorate([
    dec
], C.prototype, "accessor", null);
