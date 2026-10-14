let index = 0;
let recursive;
function objectPattern({ value = {
  [index++]: class {
    static { if (index < 2) recursive = objectPattern({}); }
    static [0] = () => {};
  }
} } = {}) { return value; }
const result = objectPattern({});
expect(index).toBe(2);
expect(result[0].name).toBe("0");
expect(recursive[1].name).toBe("1");
expect(result[0][0].name).toBe("0");

let arrayIndex = 0;
let arrayRecursive;
function arrayPattern([value = {
  [arrayIndex++]: class {
    static { if (arrayIndex < 2) arrayRecursive = arrayPattern([]); }
    static [0] = () => {};
  }
}] = []) { return value; }
const arrayResult = arrayPattern([]);
expect(arrayIndex).toBe(2);
expect(arrayResult[0].name).toBe("0");
expect(arrayRecursive[1].name).toBe("1");
expect(arrayResult[0][0].name).toBe("0");

let nestedIndex = 0;
let nestedRecursive;
function nestedPattern({ outer: { value = {
  [nestedIndex++]: class {
    static { if (nestedIndex < 2) nestedRecursive = nestedPattern({ outer: {} }); }
    static [0] = () => {};
  }
} } } = { outer: {} }) { return value; }
const nestedResult = nestedPattern({ outer: {} });
expect(nestedIndex).toBe(2);
expect(nestedResult[0].name).toBe("0");
expect(nestedRecursive[1].name).toBe("1");
expect(nestedResult[0][0].name).toBe("0");

let restIndex = 0;
let restRecursive;
function restPattern(...[{ value = {
  [restIndex++]: class {
    static { if (restIndex < 2) restRecursive = restPattern({}); }
    static [0] = () => {};
  }
} }]) { return value; }
const restResult = restPattern({});
expect(restIndex).toBe(2);
expect(restResult[0].name).toBe("0");
expect(restRecursive[1].name).toBe("1");
expect(restResult[0][0].name).toBe("0");

let keyIndex = 0;
let keyRecursive;
function computedPattern({ [Object.values({
  [keyIndex++]: class {
    static { if (keyIndex < 2) keyRecursive = computedPattern({ 1: "inner" }); }
    static [0] = () => {};
  }
})[0].name]: value }) { return value; }
const keyResult = computedPattern({ 0: "outer" });
expect(keyIndex).toBe(2);
expect(keyResult).toBe("outer");
expect(keyRecursive).toBe("inner");

function parenthesized(C = (((class {
  static value = 0;
  static { this.seen = this.name; }
})))) { return C; }
const Parenthesized = parenthesized();
expect([Parenthesized.name, Parenthesized.seen]).toEqual(["C", "C"]);

function nestedClassDefault(Outer = ((class {
  static Inner = class {
    static value = 0;
    static { this.seen = this.name; }
  };
}))) { return Outer; }
const NestedOuter = nestedClassDefault();
expect(NestedOuter.name).toBe("Outer");
expect([NestedOuter.Inner.name, NestedOuter.Inner.seen]).toEqual(["Inner", "Inner"]);

function noTemporaryDefault(C = (class {
  static {}
  static value = 1;
})) { return C; }
function bodyDefault(callback = function inner(C = (class {
  static {}
  static value = 2;
})) { return C; }) { return callback(); }
expect([noTemporaryDefault().name, noTemporaryDefault().value]).toEqual(["C", 1]);
expect([bodyDefault().name, bodyDefault().value]).toEqual(["C", 2]);

let SwitchClass;
switch (0) {
  case ({ [0]: class { static {} } }[0], 0):
    SwitchClass = { [0]: class { static {} } }[0];
    break;
}
expect(SwitchClass.name).toBe("0");
