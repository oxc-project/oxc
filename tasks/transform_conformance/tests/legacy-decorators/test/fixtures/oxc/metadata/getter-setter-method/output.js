class C {
    get address() {
        return "test";
    }
    set address(value) {
    }
    regularMethod() {
        return "test";
    }
}
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Function),
    babelHelpers.decorateMetadata("design:paramtypes", [])
], C.prototype, "address", null);
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Function),
    babelHelpers.decorateMetadata("design:paramtypes", [String])
], C.prototype, "address", null);
babelHelpers.decorate([
    dec,
    babelHelpers.decorateMetadata("design:type", Function),
    babelHelpers.decorateMetadata("design:paramtypes", []),
    babelHelpers.decorateMetadata("design:returntype", String)
], C.prototype, "regularMethod", null);
