function dec(target, key, descriptor) {}
var DeclaredBefore = /* @__PURE__ */ function(DeclaredBefore) {
  DeclaredBefore["A"] = "a";
  return DeclaredBefore;
}(DeclaredBefore || {});
export let ExportedBefore = /* @__PURE__ */ function(ExportedBefore) {
  ExportedBefore[ExportedBefore["A"] = 1] = "A";
  return ExportedBefore;
}({});
export class Foo {
  declaredBefore;
  exportedBefore;
  stringProp;
  numberProp;
  autoProp;
  mixedProp;
  constProp;
  method(param) {
    return 1;
  }
}
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], Foo.prototype, "declaredBefore", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "exportedBefore", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], Foo.prototype, "stringProp", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "numberProp", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "autoProp", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Object)], Foo.prototype, "mixedProp", void 0);
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], Foo.prototype, "constProp", void 0);
babelHelpers.decorate([
  dec,
  babelHelpers.decorateMetadata("design:type", Function),
  babelHelpers.decorateMetadata("design:paramtypes", [String]),
  babelHelpers.decorateMetadata("design:returntype", Number)
], Foo.prototype, "method", null);
export let StringEnum = /* @__PURE__ */ function(StringEnum) {
  StringEnum["A"] = "a";
  StringEnum["B"] = "b";
  return StringEnum;
}({});
var NumberEnum = /* @__PURE__ */ function(NumberEnum) {
  NumberEnum[NumberEnum["A"] = 1] = "A";
  NumberEnum[NumberEnum["B"] = 2] = "B";
  return NumberEnum;
}(NumberEnum || {});
export let AutoIncrementEnum = /* @__PURE__ */ function(AutoIncrementEnum) {
  AutoIncrementEnum[AutoIncrementEnum["A"] = 0] = "A";
  AutoIncrementEnum[AutoIncrementEnum["B"] = 1] = "B";
  return AutoIncrementEnum;
}({});
var MixedEnum = /* @__PURE__ */ function(MixedEnum) {
  MixedEnum["A"] = "a";
  MixedEnum[MixedEnum["B"] = 1] = "B";
  return MixedEnum;
}(MixedEnum || {});
var ConstEnum = /* @__PURE__ */ function(ConstEnum) {
  ConstEnum["A"] = "a";
  return ConstEnum;
}(ConstEnum || {});
