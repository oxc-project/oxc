var _FIELD_NAME;
const FIELD_NAME = "myField";
const decoratedKeys = [];
function Field() {
  return (target, key) => {
    decoratedKeys.push(key);
  };
}
class MyModel {
  static {
    _FIELD_NAME = FIELD_NAME;
  }
}
babelHelpers.decorate([Field()], MyModel.prototype, _FIELD_NAME, void 0);
expect(decoratedKeys).toEqual(["myField"]);
const instance = new MyModel();
expect(Object.prototype.hasOwnProperty.call(instance, "myField")).toBe(false);
