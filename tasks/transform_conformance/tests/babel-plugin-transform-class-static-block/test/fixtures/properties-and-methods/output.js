var _staticBlock, _staticBlock2;
function foo() {}
export let C = (class C {
  #_ = 1;
  static #_2 = 2;
  #_3() {}
  static #_4() {}
  #__5_accessor_storage = 5;
  get #_5() {
    return this.#__5_accessor_storage;
  }
  set #_5(value) {
    this.#__5_accessor_storage = value;
  }
  static #__6_accessor_storage = 6;
  static get #_6() {
    return C.#__6_accessor_storage;
  }
  static set #_6(value) {
    C.#__6_accessor_storage = value;
  }
  _7 = 7;
  static _8 = ((_value) => (_staticBlock = () => (foo(), this), _value))(8);
  _9() {}
  static _10() {}
}, _staticBlock());
export let D = (class D {
  #_;
  #_2() {}
  static #_3() {}
  #__4_accessor_storage;
  get #_4() {
    return this.#__4_accessor_storage;
  }
  set #_4(value) {
    this.#__4_accessor_storage = value;
  }
  _5;
  static #_5 = _staticBlock2 = () => (foo(), this);
}, _staticBlock2());
