var StringEnum = /* @__PURE__ */ function(StringEnum) {
  StringEnum["foo"] = "string";
  StringEnum["bar"] = "another";
  return StringEnum;
}(StringEnum || {});

var TemplateStringEnum = /* @__PURE__ */ function(TemplateStringEnum) {
  TemplateStringEnum["template"] = "template literal";
  TemplateStringEnum["mixed"] = "prefix_suffix";
  return TemplateStringEnum;
}(TemplateStringEnum || {});

var NumberEnum = /* @__PURE__ */ function(NumberEnum) {
  NumberEnum[NumberEnum["a"] = 1] = "a";
  NumberEnum[NumberEnum["b"] = 2] = "b";
  return NumberEnum;
}(NumberEnum || {});

var UnaryEnum = /* @__PURE__ */ function(UnaryEnum) {
  UnaryEnum[UnaryEnum["negative"] = -1] = "negative";
  UnaryEnum[UnaryEnum["positive"] = 2] = "positive";
  UnaryEnum[UnaryEnum["bitwise"] = -4] = "bitwise";
  return UnaryEnum;
}(UnaryEnum || {});

function getString() {
  return "string";
}

var UnaryOtherEnum = /* @__PURE__ */ function(UnaryOtherEnum) {
  UnaryOtherEnum[UnaryOtherEnum["negative"] = -getString()] = "negative";
  UnaryOtherEnum[UnaryOtherEnum["positive"] = +getString()] = "positive";
  UnaryOtherEnum[UnaryOtherEnum["bitwise"] = ~getString()] = "bitwise";
  return UnaryOtherEnum;
}(UnaryOtherEnum || {});

var AutoIncrementEnum = /* @__PURE__ */ function(AutoIncrementEnum) {
  AutoIncrementEnum[AutoIncrementEnum["first"] = 0] = "first";
  AutoIncrementEnum[AutoIncrementEnum["second"] = 1] = "second";
  AutoIncrementEnum[AutoIncrementEnum["third"] = 2] = "third";
  return AutoIncrementEnum;
}(AutoIncrementEnum || {});

var MixedEnum = /* @__PURE__ */ function(MixedEnum) {
  MixedEnum["str"] = "string";
  MixedEnum[MixedEnum["num"] = 1] = "num";
  return MixedEnum;
}(MixedEnum || {});

var ComputedEnum = /* @__PURE__ */ function(ComputedEnum) {
  ComputedEnum[ComputedEnum["computed"] = Math.PI] = "computed";
  ComputedEnum[ComputedEnum["expression"] = 3] = "expression";
  return ComputedEnum;
}(ComputedEnum || {});

function decorate(target, property) {}

export class Foo {
  stringProp;
  templateProp;
  numberProp;
  unaryProp;
  unaryOtherProp;
  autoProp;
  mixedProp;
  computedProp;
  method(param) {
    return NumberEnum.a;
  }
}

babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", String)], Foo.prototype, "stringProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", String)], Foo.prototype, "templateProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "numberProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "unaryProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "unaryOtherProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", Number)], Foo.prototype, "autoProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", Object)], Foo.prototype, "mixedProp", void 0);
babelHelpers.decorate([decorate, babelHelpers.decorateMetadata("design:type", Object)], Foo.prototype, "computedProp", void 0);
babelHelpers.decorate([
  decorate,
  babelHelpers.decorateMetadata("design:type", Function),
  babelHelpers.decorateMetadata("design:paramtypes", [String]),
  babelHelpers.decorateMetadata("design:returntype", Number)
], Foo.prototype, "method", null);
