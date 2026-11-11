"use strict";

const foo = () => function () {
  return this === undefined;
};
console.log(((foo?.()))());

function Foo() {
  return ((new.target?.()))();
}

function callMissing(a, arg) {
  return (a?.b)(arg())?.c;
}

function callOverriddenBind() {
  const b = function () {
    return { c: this };
  };
  b.bind = undefined;
  b.call = undefined;
  const a = { b };
  return (a?.b)()?.c;
}

function callComputed(a, key, ...args) {
  return ((a?.[key()]))(...args)?.c;
}

function callGetter() {
  let a = {
    get b() {
      a = {};
      return function () {
        return { c: this };
      };
    },
  };
  return (a?.b)()?.c;
}

function callDefault(a, result = (a?.b)()) {
  return result;
}
