var _staticBlock3;
let _key, _key3, _key5, _key7, _key9, _key11;
let index = 0;
let recursive;
function objectPattern({ value = { [_key = babelHelpers.toPropertyKey(index++)]: (() => {
  let _key2;
  return ((_name, _object) => _object[_name])(_key, { [_key]: class {
    static [_key2 = babelHelpers.toPropertyKey(0)] = ((() => {
      if (index < 2) recursive = objectPattern({});
    })(), babelHelpers.setFunctionName(() => {}, _key2));
  } });
})() } } = {}) {
  return value;
}
const result = objectPattern({});
let arrayIndex = 0;
let arrayRecursive;
function arrayPattern([value = { [_key3 = babelHelpers.toPropertyKey(arrayIndex++)]: (() => {
  let _key4;
  return ((_name2, _object2) => _object2[_name2])(_key3, { [_key3]: class {
    static [_key4 = babelHelpers.toPropertyKey(0)] = ((() => {
      if (arrayIndex < 2) arrayRecursive = arrayPattern([]);
    })(), babelHelpers.setFunctionName(() => {}, _key4));
  } });
})() }] = []) {
  return value;
}
const arrayResult = arrayPattern([]);
let nestedIndex = 0;
let nestedRecursive;
function nestedPattern({ outer: { value = { [_key5 = babelHelpers.toPropertyKey(nestedIndex++)]: (() => {
  let _key6;
  return ((_name3, _object3) => _object3[_name3])(_key5, { [_key5]: class {
    static [_key6 = babelHelpers.toPropertyKey(0)] = ((() => {
      if (nestedIndex < 2) nestedRecursive = nestedPattern({ outer: {} });
    })(), babelHelpers.setFunctionName(() => {}, _key6));
  } });
})() } } } = { outer: {} }) {
  return value;
}
const nestedResult = nestedPattern({ outer: {} });
let restIndex = 0;
let restRecursive;
function restPattern(...[{ value = { [_key7 = babelHelpers.toPropertyKey(restIndex++)]: (() => {
  let _key8;
  return ((_name4, _object4) => _object4[_name4])(_key7, { [_key7]: class {
    static [_key8 = babelHelpers.toPropertyKey(0)] = ((() => {
      if (restIndex < 2) restRecursive = restPattern({});
    })(), babelHelpers.setFunctionName(() => {}, _key8));
  } });
})() } }]) {
  return value;
}
const restResult = restPattern({});
let keyIndex = 0;
let keyRecursive;
function computedPattern({ [Object.values({ [_key9 = babelHelpers.toPropertyKey(keyIndex++)]: (() => {
  let _key10;
  return ((_name5, _object5) => _object5[_name5])(_key9, { [_key9]: class {
    static [_key10 = babelHelpers.toPropertyKey(0)] = ((() => {
      if (keyIndex < 2) keyRecursive = computedPattern({ 1: "inner" });
    })(), babelHelpers.setFunctionName(() => {}, _key10));
  } });
})() })[0].name]: value }) {
  return value;
}
const keyResult = computedPattern({ 0: "outer" });
function parenthesized(C = (() => {
  var _staticBlock;
  return { "C": class {
    static value = ((_value) => (_staticBlock = () => (this.seen = this.name, this), _value))(0);
  } }["C"], _staticBlock();
})()) {
  return C;
}
const Parenthesized = parenthesized();
function nestedClassDefault(Outer = (() => {
  var _staticBlock2;
  return { "Outer": class {
    static Inner = ({ "Inner": class {
      static value = ((_value2) => (_staticBlock2 = () => (this.seen = this.name, this), _value2))(0);
    } }["Inner"], _staticBlock2());
  } }["Outer"];
})()) {
  return Outer;
}
const NestedOuter = nestedClassDefault();
function noTemporaryDefault(C = class {
  static value = ((() => {})(), 1);
}) {
  return C;
}
function bodyDefault(callback = function inner(C = class {
  static value = ((() => {})(), 2);
}) {
  return C;
}) {
  return callback();
}
let SwitchClass;
switch (0) {
  case { [_key11 = babelHelpers.toPropertyKey(0)]: (((_name6, _object6) => _object6[_name6])(_key11, { [_key11]: class {
    static #_ = _staticBlock3 = () => ((() => {})(), this);
  } }), _staticBlock3()) }[0], 0:
    var _staticBlock4;
    let _key12;
    SwitchClass = { [_key12 = babelHelpers.toPropertyKey(0)]: (((_name7, _object7) => _object7[_name7])(_key12, { [_key12]: class {
      static #_ = _staticBlock4 = () => ((() => {})(), this);
    } }), _staticBlock4()) }[0];
    break;
}
