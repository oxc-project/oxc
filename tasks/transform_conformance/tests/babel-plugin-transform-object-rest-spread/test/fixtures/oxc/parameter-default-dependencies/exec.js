function f({ a, ...x }, b = a) { return [b, x]; }
expect(f({ a: 1, c: 2 })).toEqual([1, { c: 2 }]);
expect(f({ a: 1 }, 3)).toEqual([3, {}]);
expect(f.length).toBe(1);

const arrow = ({ a, ...rest }, b = a) => [b, rest];
expect(arrow({ a: 1 })).toEqual([1, {}]);
expect(arrow.length).toBe(1);

const order = [];
function ordered({ a, ...rest }, b = (order.push(a), a), { c } = (order.push(b), { c: b }), ...tail) {
  return [c, tail];
}
expect(ordered({ a: 1 })).toEqual([1, []]);
expect(order).toEqual([1, 1]);
expect(ordered({ a: 1 }, 2, { c: 3 }, 4, 5)).toEqual([3, [4, 5]]);

function forward(b = a, { a, ...rest }) { return b; }
expect(() => forward(undefined, { a: 1 })).toThrow(ReferenceError);
function self({ a, ...rest }, b = b) { return b; }
expect(() => self({ a: 1 })).toThrow(ReferenceError);

function nested({ a, ...rest }, { b = a }) { return b; }
expect(nested({ a: 1 }, {})).toBe(1);
function computed({ a, ...rest }, { [a]: b }) { return b; }
expect(computed({ a: 'key' }, { key: 2 })).toBe(2);
function closure({ a, ...rest }, selected = a, read = () => a) { a = 2; return read(); }
expect(closure({ a: 1 })).toBe(2);
function immediate({ a, ...rest }, selected = a, b = (() => a)()) { return b; }
expect(immediate({ a: 1 })).toBe(1);
function earlier(read = () => a, { a, ...rest }, b = read(), selected = a) { return b; }
expect(earlier(undefined, { a: 1 })).toBe(1);

const outside = 7;
function shadowed({ a, ...rest }, b = a, read = () => outside) {
  let outside = 9;
  return [read(), outside, b, this, arguments[0]];
}
const receiver = {};
const argument = { a: 1 };
expect(shadowed.call(receiver, argument)).toEqual([7, 9, 1, receiver, argument]);
function redeclared({ a, ...rest }, selected = a, read = () => a) {
  var a = 2;
  return [a, read()];
}
expect(redeclared({ a: 1 })).toEqual([2, 1]);

function named({ a, ...rest }, selected = a, value = () => a) { return value.name; }
expect(named({ a: 1 })).toBe('value');

function lexical() {
  return (({ a, ...rest }, b = a) => [b, arguments[0], this])({ a: 1 });
}
expect(lexical.call(receiver, 8)).toEqual([1, 8, receiver]);

function unmapped({ a, ...rest }, { b = (arguments[2] = 9, a) }, c) { return [b, c, arguments[2]]; }
expect(unmapped({ a: 1 }, {}, 3)).toEqual([1, 3, 9]);
expect(unmapped.length).toBe(3);
function className({ a, ...rest }, selected = a, value = class { static label = this.name; method() { return a; } }) { return [value.name, value.label, new value().method()]; }
expect(className({ a: 1 })).toEqual(['value', 'value', 1]);
function evaluation({ a, ...rest }, selected = a, b = eval('a')) { let aBody = 2; return b; }
expect(evaluation({ a: 1 })).toBe(1);
function bodyEval({ a, ...rest }, b = a) { let hidden = 3; return eval('hidden') + b; }
expect(bodyEval({ a: 1 })).toBe(4);

function* unchangedGenerator({ a, ...rest }, b = a) { yield b; }
unchangedGenerator.prototype.custom = () => 'custom';
const iterator = unchangedGenerator({ a: 1 }, 2);
expect(Object.getPrototypeOf(iterator)).toBe(unchangedGenerator.prototype);
expect(iterator instanceof unchangedGenerator).toBe(true);
expect(iterator.custom()).toBe('custom');
expect(iterator.next().value).toBe(2);
expect(() => new unchangedGenerator({ a: 1 }, 2)).toThrow(TypeError);

async function unchangedAsync({ a, ...rest }, b = a, read = () => outside) { let outside = 2; return b; }
expect(unchangedAsync.constructor.name).toBe('AsyncFunction');
expect(() => new unchangedAsync({ a: 1 }, 2)).toThrow(TypeError);
async function asynchronous({ a, ...rest }, b = a) { return b; }
expect(asynchronous.constructor.name).toBe('AsyncFunction');
return Promise.all([
  asynchronous({ a: 1 }).then(value => expect(value).toBe(1)),
  unchangedAsync({ a: 1 }, 2).then(value => expect(value).toBe(2)),
]);
