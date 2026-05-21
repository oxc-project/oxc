import { dce, Dependency } from "mod";

let AbstractClass = class AbstractClass {
  dependency;
  constructor(dependency) {
    this.dependency = dependency;
  }
};
AbstractClass = babelHelpers.decorate([dce(), babelHelpers.decorateMetadata("design:paramtypes", [typeof Dependency === "undefined" ? Object : Dependency])], AbstractClass);

export { AbstractClass };
