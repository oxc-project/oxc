var _Decorated_decorators, _staticBlock, _staticBlock2;
let _init, _Decorated;
function keep(value) {
  return value;
}
let _value = /* @__PURE__ */ new WeakMap();
_Decorated_decorators = [[void 0, keep]];
class Decorated {}
_Decorated = Decorated;
_init = babelHelpers.decoratorStart(null);
_Decorated = Decorated = babelHelpers.decorateElement(_init, 0, "Decorated", _Decorated_decorators, Decorated);
babelHelpers.classPrivateFieldInitSpec(Decorated, _value, 1);
_Decorated.value = babelHelpers.classPrivateFieldGet2(_value, _Decorated);
babelHelpers.runInitializers(_init, 1, Decorated);
let Plain = (class Plain {
  static #value = ((_value2) => (_staticBlock = () => (this.value = this.#value, this), _value2))(2);
}, _staticBlock());
let Accessor = (class Accessor {
  #_value_accessor_storage = 3;
  get value() {
    return this.#_value_accessor_storage;
  }
  set value(value) {
    this.#_value_accessor_storage = value;
  }
  static #value = ((_value3) => (_staticBlock2 = () => (this.other = this.#value, this), _value3))(4);
}, _staticBlock2());
