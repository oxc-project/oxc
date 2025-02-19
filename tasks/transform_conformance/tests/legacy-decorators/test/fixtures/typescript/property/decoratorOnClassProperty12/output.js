class A {
    foo;
}
babelHelpers.decorate([
    dec(),
    babelHelpers.decorateMetadata("design:type", String)
], A.prototype, "foo", void 0);
