const userKey = Symbol("user method");
const observed = [];
function inspect(getClass) {
  observed.push(Object.getOwnPropertySymbols(getClass()).filter(key => key !== Symbol.metadata));
}

class Outer {
  #_computedKey = "x";
  #_computedKey3 = "y";
  *make() {
    return class C {
      static #_computedKey2() {}
      static [userKey]() {}
      @inspect static getClass() { return C; }
      static [(yield "key", this.#_computedKey)] = 1;
      static [eval("this.#_computedKey3")] = 2;
    };
  }
}

const it = new Outer().make();
expect(it.next().value).toBe("key");
const C = it.next().value;
expect(C.x).toBe(1);
expect(C.y).toBe(2);
expect(observed).toEqual([[userKey]]);
