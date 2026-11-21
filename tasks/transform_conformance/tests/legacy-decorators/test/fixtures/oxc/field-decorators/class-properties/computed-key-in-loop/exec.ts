// Each class defined in a loop keeps its own decorated computed key.
const keys: PropertyKey[] = [];
const dec = (_target: any, key: PropertyKey) => {
  keys.push(key);
};
const classes: any[] = [];
for (const n of ["a", "b"]) {
  class A {
    @dec [n] = n;
  }
  classes.push(A);
}

expect(keys).toEqual(["a", "b"]);
expect(Object.keys(new classes[0]())).toEqual(["a"]);
expect(Object.keys(new classes[1]())).toEqual(["b"]);
