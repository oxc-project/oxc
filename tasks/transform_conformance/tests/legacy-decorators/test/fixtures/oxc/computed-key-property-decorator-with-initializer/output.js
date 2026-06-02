var _FIELD_NAME;
const FIELD_NAME = "myField";
function dec(target, key) {}
class MyModel {
  [_FIELD_NAME = FIELD_NAME] = "value";
}
babelHelpers.decorate([dec], MyModel.prototype, _FIELD_NAME, void 0);
