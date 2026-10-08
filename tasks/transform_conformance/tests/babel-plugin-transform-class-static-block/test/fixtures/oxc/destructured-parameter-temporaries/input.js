let index = 0;
let recursive;
function objectPattern({ value = {
  [index++]: class {
    static { if (index < 2) recursive = objectPattern({}); }
    static [0] = () => {};
  }
} } = {}) { return value; }
const result = objectPattern({});

let arrayIndex = 0;
let arrayRecursive;
function arrayPattern([value = {
  [arrayIndex++]: class {
    static { if (arrayIndex < 2) arrayRecursive = arrayPattern([]); }
    static [0] = () => {};
  }
}] = []) { return value; }
const arrayResult = arrayPattern([]);

let nestedIndex = 0;
let nestedRecursive;
function nestedPattern({ outer: { value = {
  [nestedIndex++]: class {
    static { if (nestedIndex < 2) nestedRecursive = nestedPattern({ outer: {} }); }
    static [0] = () => {};
  }
} } } = { outer: {} }) { return value; }
const nestedResult = nestedPattern({ outer: {} });

let restIndex = 0;
let restRecursive;
function restPattern(...[{ value = {
  [restIndex++]: class {
    static { if (restIndex < 2) restRecursive = restPattern({}); }
    static [0] = () => {};
  }
} }]) { return value; }
const restResult = restPattern({});

let keyIndex = 0;
let keyRecursive;
function computedPattern({ [Object.values({
  [keyIndex++]: class {
    static { if (keyIndex < 2) keyRecursive = computedPattern({ 1: "inner" }); }
    static [0] = () => {};
  }
})[0].name]: value }) { return value; }
const keyResult = computedPattern({ 0: "outer" });

function parenthesized(C = (((class {
  static value = 0;
  static { this.seen = this.name; }
})))) { return C; }
const Parenthesized = parenthesized();

function nestedClassDefault(Outer = ((class {
  static Inner = class {
    static value = 0;
    static { this.seen = this.name; }
  };
}))) { return Outer; }
const NestedOuter = nestedClassDefault();

function noTemporaryDefault(C = (class {
  static {}
  static value = 1;
})) { return C; }
function bodyDefault(callback = function inner(C = (class {
  static {}
  static value = 2;
})) { return C; }) { return callback(); }

let SwitchClass;
switch (0) {
  case ({ [0]: class { static {} } }[0], 0):
    SwitchClass = { [0]: class { static {} } }[0];
    break;
}
