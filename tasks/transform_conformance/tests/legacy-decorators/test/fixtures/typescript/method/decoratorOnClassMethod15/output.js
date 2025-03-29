class Foo {
    prop = 1;
    foo() {
        return 0;
    }
}
babelHelpers.decorate([
    decorator,
    babelHelpers.decorateMetadata("design:type", Function),
    babelHelpers.decorateMetadata("design:paramtypes", []),
    babelHelpers.decorateMetadata("design:returntype", void 0)
], Foo.prototype, "foo", null);
