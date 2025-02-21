export class T {
  method(test) {
    return test;
  }
}
babelHelpers.decorate([first(), babelHelpers.decorateParam(0, first())], T.prototype, "method", null);
