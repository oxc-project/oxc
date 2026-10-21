const key = Symbol("");
class A {
  Computed = class {
    static [key] = function () {};
    static x = ((() => {})(), 4);
  };
  Child = class {
    static x = ((() => {})(), 1);
  };
  ["computed"] = class {
    static x = ((() => {})(), 2);
  };
}
function create(Child = class {
  static x = ((() => {})(), 3);
}) {
  return Child;
}
