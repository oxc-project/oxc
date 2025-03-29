class Foo {
    p1;
    p2;
}
babelHelpers.decorate([
    decorator(),
    babelHelpers.decorateMetadata("design:type", Object)
], Foo.prototype, "p2", void 0);
