var _staticBlock, _staticBlock3;
let _key;
let Names = (class Names {
  static asFunction = ((() => {})(), babelHelpers.setFunctionName(function() {}, "asFunction"));
  static assertedArrow = ((() => {})(), babelHelpers.setFunctionName((() => {}), "assertedArrow"));
  static satisfiesFunction = ((() => {})(), babelHelpers.setFunctionName(function() {}, "satisfiesFunction"));
  static nonNullClass = ((() => {})(), { "nonNullClass": class {
    static seen = this.name;
  } }["nonNullClass"]);
  static asClass = ((() => {})(), { "asClass": class {
    static seen = this.name;
  } }["asClass"]);
  static satisfiesClass = ((() => {})(), { "satisfiesClass": class {
    static seen = this.name;
  } }["satisfiesClass"]);
  static namedFunction = ((() => {})(), function Explicit() {});
  static nestedClass = ((_value) => (_staticBlock = () => ((() => {})(), this), _value))(((() => {})(), { "nestedClass": class {
    static seen = this.name;
  } }["nestedClass"]));
}, _staticBlock());
function defaultClass(C = (() => {
  var _staticBlock2;
  return { "C": class {
    static value = ((_value2) => (_staticBlock2 = () => (this.seen = this.name, this), _value2))(0);
  } }["C"], _staticBlock2();
})()) {
  return C;
}
const Default = defaultClass();
const objectClasses = { ObjectClass: ({ "ObjectClass": class {
  static value = ((_value3) => (_staticBlock3 = () => (this.seen = this.name, this), _value3))(0);
} }["ObjectClass"], _staticBlock3()) };
const instanceKey = "instance";
class Holder {
  [_key = babelHelpers.toPropertyKey(instanceKey)] = (() => {
    var _staticBlock4;
    return ((_name, _object) => _object[_name])(_key, { [_key]: class {
      static value = ((_value4) => (_staticBlock4 = () => (this.seen = this.name, this), _value4))(0);
    } }), _staticBlock4();
  })();
}
function* suspendedFields() {
  let _key2;
  const Holder = class _Class {
    static #_ = (babelHelpers.setFunctionName(this, "Holder"), _key2);
    [_key2 = babelHelpers.toPropertyKey(yield "key")] = (() => {
      var _staticBlock5;
      return { [_Class.#_]: class {
        static value = ((_value5) => (_staticBlock5 = () => (this.seen = this.name, this), _value5))(0);
      } }[_Class.#_], _staticBlock5();
    })();
  };
  return Holder;
}
