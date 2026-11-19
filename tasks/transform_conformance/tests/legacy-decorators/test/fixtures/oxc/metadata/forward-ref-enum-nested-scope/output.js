function dec(target, key) {}
export let NS;
(function(_NS) {
  class InNamespace {
    prop;
  }
  babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], InNamespace.prototype, "prop", void 0);
  _NS.InNamespace = InNamespace;
  let Later = /* @__PURE__ */ function(Later) {
    Later["A"] = "a";
    return Later;
  }(_NS.Later || {});
  _NS.Later = Later;
})(NS || (NS = {}));
export function inFunction() {
  class InFunction {
    prop;
  }
  babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", Number)], InFunction.prototype, "prop", void 0);
  let Later = /* @__PURE__ */ function(Later) {
    Later[Later["A"] = 1] = "A";
    return Later;
  }({});
  return InFunction;
}
{
  class InBlock {
    prop;
  }
  babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", String)], InBlock.prototype, "prop", void 0);
  let Later = /* @__PURE__ */ function(Later) {
    Later["A"] = "a";
    return Later;
  }({});
}
