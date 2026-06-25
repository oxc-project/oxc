let Example = class Example {
  constructor(count) {}
};

Example = babelHelpers.decorate(
  [
    babelHelpers.decorateParam(0, dce),
    babelHelpers.decorateMetadata("design:paramtypes", [
      typeof UnboundTypeReference === "undefined" ? Object : UnboundTypeReference,
    ]),
  ],
  Example,
);
