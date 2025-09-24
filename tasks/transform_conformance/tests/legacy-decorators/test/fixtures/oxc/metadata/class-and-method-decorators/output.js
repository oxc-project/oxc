import { singleton, log, deco, C } from "dec";
let Problem = class Problem extends C {
  run() {
    return super.run();
  }
};

babelHelpers.decorate(
  [
    deco(),
    babelHelpers.decorateMetadata("design:type", Function),
    babelHelpers.decorateMetadata("design:paramtypes", []),
    babelHelpers.decorateMetadata("design:returntype", void 0),
  ],
  Problem.prototype,
  "run",
  null,
);
Problem = babelHelpers.decorate([log("Problem"), singleton()], Problem);

export { Problem };
