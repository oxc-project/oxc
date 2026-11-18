"use strict";
const foo = () => function() {
  return this === undefined;
};
console.log((foo === null || foo === void 0 ? void 0 : foo())());
function Foo() {
  var _new$target;
  return ((_new$target = new.target) === null || _new$target === void 0 ? void 0 : _new$target())();
}
function callMissing(a, arg) {
  var _a$b, _a;
  return (_a$b = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a = a).b, _a))(arg())) === null || _a$b === void 0 ? void 0 : _a$b.c;
}
function callOverriddenBind() {
  var _a$b2, _a2;
  const b = function() {
    return { c: this };
  };
  b.bind = undefined;
  b.call = undefined;
  const a = { b };
  return (_a$b2 = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a2 = a).b, _a2))()) === null || _a$b2 === void 0 ? void 0 : _a$b2.c;
}
function callComputed(a, key, ...args) {
  var _a$key, _a3;
  return (_a$key = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a3 = a)[key()], _a3))(...args)) === null || _a$key === void 0 ? void 0 : _a$key.c;
}
function callGetter() {
  var _a$b3, _a4;
  let a = { get b() {
    a = {};
    return function() {
      return { c: this };
    };
  } };
  return (_a$b3 = (a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a4 = a).b, _a4))()) === null || _a$b3 === void 0 ? void 0 : _a$b3.c;
}
function callDefault(a, result = (() => {
  var _a5;
  return a === null || a === void 0 ? void 0 : (() => {}).call.bind((_a5 = a).b, _a5);
})()()) {
  return result;
}
