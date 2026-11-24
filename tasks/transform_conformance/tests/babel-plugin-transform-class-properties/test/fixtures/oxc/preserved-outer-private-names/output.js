@decorate class Outer {
  #value = 1;
  #method() {}
  makeInner() {
    var _own;
    return _own = /* @__PURE__ */ new WeakMap(), class Inner {
      constructor() {
        babelHelpers.classPrivateFieldInitSpec(this, _own, 2);
      }
      read(outer) {
        return [
          outer.#value,
          outer.#method(),
          outer?.#value,
          outer.#method?.(),
          #value in outer
        ];
      }
      write(outer, values) {
        outer.#value = 3;
        outer.#value += 1;
        outer.#value++;
        [outer.#value] = values;
        return babelHelpers.classPrivateFieldGet2(_own, this);
      }
    };
  }
}
class AccessorOuter {
  accessor value = 1;
  #value = 2;
  makeInner() {
    var _own2;
    return _own2 = /* @__PURE__ */ new WeakMap(), class Inner {
      constructor() {
        babelHelpers.classPrivateFieldInitSpec(this, _own2, 3);
      }
      read(outer) {
        return outer.#value + babelHelpers.classPrivateFieldGet2(_own2, this);
      }
    };
  }
}

