let rest;
const values = [1, 2];
const result = ([...{ ...rest }] = values);
expect(result).toBe(values);
expect(rest).toEqual({ 0: 1, 1: 2 });

let source = values;
let sourceCalls = 0;
let setterCalls = 0;
function getSource() {
  sourceCalls++;
  return source;
}
const target = {
  set value(value) {
    setterCalls++;
    source = null;
    rest = value;
  },
};
expect(([...{ ...target.value }] = getSource())).toBe(values);
expect(sourceCalls).toBe(1);
expect(setterCalls).toBe(1);
expect(source).toBe(null);
expect(rest).toEqual({ 0: 1, 1: 2 });

let first, second;
[first, ...{ 0: second, ...rest }] = [1, 2, 3];
expect(first).toBe(1);
expect(second).toBe(2);
expect(rest).toEqual({ 1: 3 });

[...[{ ...rest }]] = [{ value: 1 }];
expect(rest).toEqual({ value: 1 });

[...[{ ...rest } = { value: 2 }]] = [];
expect(rest).toEqual({ value: 2 });

const log = [];
function key() {
  log.push("key");
  return "0";
}
function fallback() {
  log.push("default");
  return 42;
}
[...{ [key()]: first = fallback(), ...rest }] = [undefined, 2];
expect(first).toBe(42);
expect(rest).toEqual({ 1: 2 });
expect(log).toEqual(["key", "default"]);

const iterations = [];
for ([...{ ...rest }] of [[1, 2], [3]]) {
  iterations.push(rest);
}
expect(iterations).toEqual([{ 0: 1, 1: 2 }, { 0: 3 }]);

for ([...{ ...rest }] in { ab: 0 }) {
  expect(rest).toEqual({ 0: "a", 1: "b" });
}
