import { BoundTypeReference } from "./output";

console.log(BoundTypeReference);

let Example = class Example {
  constructor(count) {}
  prop = 1;
};

Example = babelHelpers.decorate(
  [
    babelHelpers.decorateParam(0, dce),
    babelHelpers.decorateMetadata("design:paramtypes", [
      typeof BoundTypeReference === "undefined" ? Object : BoundTypeReference,
    ]),
  ],
  Example,
);
