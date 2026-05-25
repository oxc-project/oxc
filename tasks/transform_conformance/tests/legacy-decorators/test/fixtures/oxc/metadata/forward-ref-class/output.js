class Source {
  laterRef;
}
babelHelpers.decorate([dec, babelHelpers.decorateMetadata("design:type", typeof LaterClass === "undefined" ? Object : LaterClass)], Source.prototype, "laterRef", void 0);
class LaterClass {
  tag = "later";
}
