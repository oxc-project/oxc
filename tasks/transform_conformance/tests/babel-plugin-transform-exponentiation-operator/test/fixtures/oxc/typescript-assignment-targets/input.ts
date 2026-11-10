import { receiver, replaceReceiver } from "receiver";

let key = 0;
function getObject() { return { x: 2 }; }
function getKey() { return "x"; }

function assignments(value: number) {
  value! **= 2;
  (value as number) **= 2;
  (value satisfies number) **= 2;
  (<number>value) **= 2;
  ((value) as number)! **= 2;
  (value) **= 2;
  unbound! **= 2;
  (unbound as number) **= 2;
  (unbound satisfies number) **= 2;
  (<number>unbound) **= 2;
  ((unbound) as number)! **= 2;
  (unbound) **= 2;
  getObject().x! **= 2;
  (getObject().x as number) **= 2;
  (getObject().x satisfies number) **= 2;
  (<number>getObject().x) **= 2;
  ((getObject().x) as number)! **= 2;
  (getObject().x) **= 2;
  getObject()[++key]! **= 2;
  (getObject()[++key] as number) **= 2;
  (getObject()[++key] satisfies number) **= 2;
  (<number>getObject()[++key]) **= 2;
  ((getObject()[++key]) as number)! **= 2;
  (getObject()[++key]) **= 2;
}

class Base {
  get x() { return 2; }
  set x(value: number) {}
}
class Derived extends Base {
  assignments() {
    super.x! **= 2;
    (super.x as number) **= 2;
    (super.x satisfies number) **= 2;
    (<number>super.x) **= 2;
    ((super.x) as number)! **= 2;
    (super.x) **= 2;
    super[getKey()]! **= 2;
    (super[getKey()] as number) **= 2;
    (super[getKey()] satisfies number) **= 2;
    (<number>super[getKey()]) **= 2;
    ((super[getKey()]) as number)! **= 2;
    (super[getKey()]) **= 2;
  }
}

function reassignedReceiver() {
  const original = { x: 2 }, replacement = { x: 3 };
  let obj = original;
  (obj[(obj = replacement, "x")] as number) **= 2;
  return [original.x, replacement.x];
}

function reassignedReceiverInKeyCall() {
  const original = { x: 2 }, replacement = { x: 3 };
  let obj = original;
  function key() { obj = replacement; return "x"; }
  obj[key()] **= 2;
  return [original.x, replacement.x];
}

function reassignedReceiverInEval() {
  const original = { x: 2 }, replacement = { x: 3 };
  let obj = original;
  (obj[(eval("obj = replacement"), "x")] as number) **= 2;
  return [original.x, replacement.x];
}

function importedReceiver() {
  (receiver[replaceReceiver()] as number) **= 2;
}

function literalKey(obj: { x: number }) {
  (obj["x"] as number) **= 2;
}
