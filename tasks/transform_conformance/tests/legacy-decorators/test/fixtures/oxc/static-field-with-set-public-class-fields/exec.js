const seen = [];
function dec(target, key) {
  seen.push([target, key]);
}

class A {
  @dec static y = 1;
  @dec z = 3;
}

expect(seen).toContainEqual([A, "y"]);
expect(seen).toContainEqual([A.prototype, "z"]);
