const key = Symbol("");
class A {
  Computed = class { static [key] = function () {}; static {} static x = 4; };
  Child = class { static {} static x = 1; };
  ["computed"] = class { static {} static x = 2; };
}
function create(Child = class { static {} static x = 3; }) {
  return Child;
}
const a = new A();
expect([a.Child.name, a.Child.x]).toEqual(["Child", 1]);
expect([a.computed.name, a.computed.x]).toEqual(["computed", 2]);
const child = create();
expect([child.name, child.x]).toEqual(["Child", 3]);
expect([a.Computed.name, a.Computed[key].name, a.Computed.x]).toEqual(["Computed", "[]", 4]);
