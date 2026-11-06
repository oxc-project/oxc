const receivers = [];
let evaluations = 0;
let obj = {
  name: "original",
  get dec() {
    obj = { name: "reassigned" };
    return function () { receivers.push(this.name); };
  },
};
function getObject() { evaluations++; return obj; }
class C { @(getObject().dec) method() {} }
expect(evaluations).toBe(1);
expect(receivers).toEqual(["original"]);

class Base {
  dec() { receivers.push(this); }
}
class Derived extends Base {
  #dec() { receivers.push(this); }
  make() {
    const C = @(super.dec) class {
      @(super.dec) method() {}
    };
    const D = @(this.#dec) class {};
    return [C, D];
  }
}
const derived = new Derived();
derived.make();
expect(receivers.slice(1)).toEqual([derived, derived, derived]);

let fakeCalled = false;
const fake = { call() { fakeCalled = true; } };
expect(() => { @fake class C {} }).toThrow(TypeError);
expect(fakeCalled).toBe(false);

const optional = {
  dec(value) {
    expect(this).toBe(optional);
    return value;
  },
};
@(optional?.dec)
class Optional {
  @(optional?.dec) method() {}
}
expect(new Optional()).toBeInstanceOf(Optional);
const holder = { child: optional };
@(holder?.child.dec)
class NestedOptional {}
expect(new NestedOptional()).toBeInstanceOf(NestedOptional);

let laterEvaluations = 0;
function later() {
  laterEvaluations++;
  return value => value;
}
expect(() => {
  @(null?.child.dec)
  @later()
  class Missing {}
}).toThrow(TypeError);
expect(laterEvaluations).toBe(1);
