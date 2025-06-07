import { dce, Dependency } from "mod";

var _ref;
let AbstractClass = class AbstractClass {
  constructor(dependency) {
    this.dependency = dependency;
  }
};
AbstractClass = babelHelpers.decorate([dce(), babelHelpers.decorateMetadata("design:paramtypes", [typeof (_ref = typeof Dependency !== "undefined" && Dependency) === "function" ? _ref : Object])], AbstractClass);

export { AbstractClass };
