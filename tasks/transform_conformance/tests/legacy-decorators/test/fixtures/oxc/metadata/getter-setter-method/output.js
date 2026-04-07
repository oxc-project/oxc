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

class UntypedGetter {
  get myProp() {
    return "hello";
  }
}

babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Object),
  babelHelpers.decorateMetadata("design:paramtypes", [])
], UntypedGetter.prototype, "myProp", null);

class UntypedSetter {
  set myProp(value) {}
}

babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Object),
  babelHelpers.decorateMetadata("design:paramtypes", [Object])
], UntypedSetter.prototype, "myProp", null);

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
