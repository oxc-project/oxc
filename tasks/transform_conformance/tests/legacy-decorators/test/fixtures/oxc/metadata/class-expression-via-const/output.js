const C = class {
  value = 1;
};
class Source {
  x;
}
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", typeof C === "undefined" ? Object : C)], Source.prototype, "x", void 0);
