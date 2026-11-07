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
function *generator({ a, ...rest }, b = a) { yield b; }
expect(generator({ a: 1 }).next().value).toBe(1);

const callEvents = [];
function* callTime({ a, ...rest }, selected = a, read = (callEvents.push('default'), () => a)) { yield read(); }
const iterator = callTime({ a: 1 });
expect(callEvents).toEqual(['default']);
expect(iterator.next().value).toBe(1);
callTime({ a: 1 }).return();
expect(callEvents).toEqual(['default', 'default']);

function* argumentIdentity({ a, ...rest }, b = a, saved = arguments) { yield saved === arguments; yield arguments.callee; }
const identityIterator = argumentIdentity({ a: 1 });
expect(identityIterator.next().value).toBe(true);
expect(() => identityIterator.next()).toThrow(TypeError);
function* argumentBinding({ a, ...rest }, b = a, read = () => arguments) { arguments = ['changed']; yield read() === arguments; }
expect(argumentBinding({ a: 1 }).next().value).toBe(true);

function* nestedArguments({ a, ...rest }, b = a) {
  function inner(c = arguments[0]) { return c; }
  yield [inner(), inner(3)];
}
expect(nestedArguments({ a: 1 }).next().value).toEqual([undefined, 3]);
const evalReceiver = { marker: 7, *read({ a, ...rest }, b = a) { yield [eval('this.marker'), eval('arguments[0].a')]; } };
expect(evalReceiver.read({ a: 1 }, 1).next().value).toEqual([7, 1]);

function* evalArguments({ a, ...rest }, b = a, saved = arguments) { yield eval('arguments') === saved; }
expect(evalArguments({ a: 1 }, 1).next().value).toBe(true);

const inherited = { value() { return this.marker; } };
const generatorReceiver = {
  __proto__: inherited,
  marker: 4,
  *read({ a, ...rest }, selected = a, read = () => a, other = () => outside) {
    let outside = 2;
    yield [super.value(), this.marker, arguments[0].a, read(), other()];
  }
};
expect(generatorReceiver.read({ a: 1 }).next().value).toEqual([4, 4, 1, 1, 7]);

function bodyEval({ a, ...rest }, b = a) { let hidden = 3; return eval('hidden') + b; }
expect(bodyEval({ a: 1 })).toBe(4);

const asyncEvents = [];
async function* asyncGenerator({ a, ...rest }, selected = a, read = (asyncEvents.push('default'), () => a)) { yield [read(), this.marker, arguments[0].a]; }
const asyncIterator = asyncGenerator.call({ marker: 3 }, { a: 1 });
expect(asyncEvents).toEqual(['default']);

const settlement = [];
async function shadowAsync({ a, ...rest }, selected = a, read = () => a, other = () => outside) { let outside = 2; return 3; }
const settled = shadowAsync({ a: 1 }).then(() => settlement.push('function'));
const tick = Promise.resolve().then(() => settlement.push('tick'));
async function rejection({ a, ...rest }, selected = a, read = () => a, b = (() => { throw new Error('default'); })()) { let outside = 2; return eval('outside'); }
let rejected;
expect(() => { rejected = rejection({ a: 1 }); }).not.toThrow();
async function asynchronous({ a, ...rest }, b = a) { return b; }
return Promise.all([
  asyncIterator.next().then(value => expect(value).toEqual({ value: [1, 3, 1], done: false })),
  asynchronous({ a: 1 }).then(value => expect(value).toBe(1)),
  Promise.all([settled, tick]).then(() => expect(settlement).toEqual(['function', 'tick'])),
  expect(rejected).rejects.toThrow('default'),
]);
