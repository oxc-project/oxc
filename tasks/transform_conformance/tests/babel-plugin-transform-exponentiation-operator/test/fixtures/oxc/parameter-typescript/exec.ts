let depth = 0;
const outer: Record<number, number> = { 1: 3 };
const inner: Record<number, number> = { 1: 5 };
function key() {
  if (depth++ === 0) recursive(inner);
  return 1;
}
function recursive(object: Record<number, number>, value = object[key()]! **= 2) {
  return value;
}
expect(recursive(outer)).toBe(9);
expect([outer[1], inner[1], depth]).toEqual([9, 25, 2]);

depth = 0;
outer[1] = 3;
inner[1] = 5;
function arrowKey() {
  if (depth++ === 0) arrow(inner);
  return 1;
}
const arrow = (object: Record<number, number>, value = (object[arrowKey()] as number) **= 2) => value;
expect(arrow(outer)).toBe(9);
expect([outer[1], inner[1], depth]).toEqual([9, 25, 2]);
