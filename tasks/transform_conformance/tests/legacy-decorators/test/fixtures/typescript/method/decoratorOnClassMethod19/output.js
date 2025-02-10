// @target: esnext, es2022, es2015
// @experimentaldecorators: true
// @emitdecoratormetadata: true
class C1 {
    #x;
    y() { }
    static {
        babelHelpers.decorate([
            decorator((x) => x.#x),
            __metadata("design:type", Function),
            __metadata("design:paramtypes", []),
            __metadata("design:returntype", void 0)
        ], C1.prototype, "y", null);
    }
}
class C2 {
    #x;
    y(p) { }
    static {
        babelHelpers.decorate([
            babelHelpers.decorateParam(0, decorator((x) => x.#x)),
            __metadata("design:type", Function),
            __metadata("design:paramtypes", [Object]),
            __metadata("design:returntype", void 0)
        ], C2.prototype, "y", null);
    }
}
