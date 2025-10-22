class Getter {
  get address() {
    return "test";
  }
  regularMethod() {
    return "test";
  }
}

babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", String),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], Getter.prototype, "address", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Function),
  babelHelpers.decorateMetadata("design:paramtypes", []),
  babelHelpers.decorateMetadata("design:returntype", String)
], Getter.prototype, "regularMethod", null);

class Setter {
  set address(value) {}
  regularMethod() {
    return "test";
  }
}

babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Number),
  babelHelpers.decorateMetadata("design:paramtypes", [Number])
], Setter.prototype, "address", null);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Function),
  babelHelpers.decorateMetadata("design:paramtypes", []),
  babelHelpers.decorateMetadata("design:returntype", String)
], Setter.prototype, "regularMethod", null);
