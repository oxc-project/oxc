const states = [];
let calls = 0;

class C {
  static {
    states.push(#field in this, #method in this, #getter in this, #setter in this);
    states.push(#field in (++calls, this));
    expect(() => #field in (++calls, null)).toThrow(TypeError);
  }

  static #field = (states.push(#field in this), undefined);

  static {
    states.push(#field in this, #empty in this, #zero in this);
  }

  static #empty;
  static #zero = 0;

  static {
    states.push(#empty in this, #zero in this, #field in {});
    expect(() => #field in 0).toThrow(TypeError);
  }

  static #method() {}
  static get #getter() {
    throw new Error("Unexpected getter call");
  }
  static set #setter(value) {
    throw new Error("Unexpected setter call");
  }
}

expect(states).toEqual([
  false, true, true, true, false, false, true, false, false, true, true, false,
]);
expect(calls).toBe(2);
