function f({ a, ...rest }, b = a, let = 3) { return { let, b }; }
expect(f({ a: 1 })).toEqual({ let: 3, b: 1 });
expect(f({ a: 1 }, 2, 4)).toEqual({ let: 4, b: 2 });
expect(f.length).toBe(1);
function named({ a, ...rest }, b = a, let = () => a) { return let.name; }
expect(named({ a: 1 })).toBe('let');
function destructured({ a, ...rest }, b = a, { let = 3 } = {}) { return let; }
expect(destructured({ a: 1 })).toBe(3);
function rest({ a, ...other }, b = a, ...let) { return let; }
expect(rest({ a: 1 }, 2, 3, 4)).toEqual([3, 4]);
function before(let, { a, ...rest }, b = a) { return let; }
expect(before(3, { a: 1 })).toBe(3);
expect(before.length).toBe(2);
function tdz({ a, ...rest }, b = a, let = let) { return let; }
expect(() => tdz({ a: 1 })).toThrow(ReferenceError);
function nested({ a, ...rest }, b = a, { let = function () {} } = {}) { return let.name; }
function array({ a, ...rest }, b = a, [let = () => a] = []) { return let.name; }
function klass({ a, ...rest }, b = a, { let = class { static inferred() { return this.name; } } } = {}) { return [let.name, let.inferred()]; }
function assignments({ a, ...rest }, b = a, let) {
  let = () => a;
  const names = [let.name];
  let = null;
  let ||= function () {};
  names.push(let.name);
  let &&= class {};
  names.push(let.name);
  let = null;
  let ??= () => a;
  names.push(let.name);
  [let = function () {}] = [];
  names.push(let.name);
  ({ x: let = class {} } = {});
  names.push(let.name);
  ({ let = () => a } = {});
  names.push(let.name);
  ({ let } = { let: 4 });
  return [names, let];
}
expect(nested({ a: 1 })).toBe('let');
expect(array({ a: 1 })).toBe('let');
expect(klass({ a: 1 })).toEqual(['let', 'let']);
expect(assignments({ a: 1 })).toEqual([Array(7).fill('let'), 4]);
