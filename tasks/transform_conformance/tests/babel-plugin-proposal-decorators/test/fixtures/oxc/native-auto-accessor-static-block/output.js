var _classPrivateFieldGet = require("@oxc-project/runtime/helpers/classPrivateFieldGet2");
var _classPrivateFieldInitSpec = require("@oxc-project/runtime/helpers/classPrivateFieldInitSpec");
var _decoratorStart = require("@oxc-project/runtime/helpers/decoratorStart");
var _decorateElement = require("@oxc-project/runtime/helpers/decorateElement");
var _runInitializers = require("@oxc-project/runtime/helpers/runInitializers");
var _Decorated_decorators, _staticBlock, _staticBlock2;
let _init, _Decorated;
function keep(value) {
  return value;
}
let _value = /* @__PURE__ */ new WeakMap();
_Decorated_decorators = [[void 0, keep]];
class Decorated {}
_Decorated = Decorated;
_init = _decoratorStart(null);
_Decorated = Decorated = _decorateElement(_init, 0, "Decorated", _Decorated_decorators, Decorated);
_classPrivateFieldInitSpec(Decorated, _value, 1);
_Decorated.value = _classPrivateFieldGet(_value, _Decorated);
_runInitializers(_init, 1, Decorated);
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
