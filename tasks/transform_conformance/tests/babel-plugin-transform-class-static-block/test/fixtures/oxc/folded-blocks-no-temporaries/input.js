const key = Symbol("");
class A {
  Computed = class { static [key] = function () {}; static {} static x = 4; };
  Child = class { static {} static x = 1; };
  ["computed"] = class { static {} static x = 2; };
}
function create(Child = class { static {} static x = 3; }) {
  return Child;
}
