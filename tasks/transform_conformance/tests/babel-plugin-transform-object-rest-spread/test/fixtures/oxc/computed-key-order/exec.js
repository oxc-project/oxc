const log = [];
let value, rest;
function key() {
  log.push("key");
  return "a";
}
function source() {
  log.push("rhs");
  return { a: 1, b: 2 };
}
const result = ({ [key()]: value, ...rest } = source());
expect(log).toEqual(["rhs", "key"]);
expect(result).toEqual({ a: 1, b: 2 });
expect(value).toBe(1);
expect(rest).toEqual({ b: 2 });

log.length = 0;
function* values() {
  log.push("iterator");
  yield undefined;
  yield 2;
  log.push("done");
}
function iterable() {
  log.push("rhs");
  return values();
}
function arrayKey() {
  log.push("key");
  return 0;
}
function fallback() {
  log.push("default");
  return 42;
}
[...{ [arrayKey()]: value = fallback(), ...rest }] = iterable();
expect(log).toEqual(["rhs", "iterator", "done", "key", "default"]);
expect(value).toBe(42);
expect(rest).toEqual({ 1: 2 });

log.length = 0;
false && ({ [key()]: value, ...rest } = source());
expect(log).toEqual([]);

function fail() {
  log.push("rhs");
  throw new Error("rhs");
}
let caught;
try {
  ({ [key()]: value, ...rest } = fail());
} catch (error) {
  caught = error;
}
expect(caught.message).toBe("rhs");
expect(log).toEqual(["rhs"]);
