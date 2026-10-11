let depth = 0;
const seen = [];
class Outer {
  Child = class {
    static id = depth++;
    static child = depth < 2 ? new Outer().Child : null;
    static { seen.push(this.id); }
  };
}
const result = new Outer();
expect(seen).toEqual([1, 0]);
expect(result.Child.id).toBe(0);
expect(result.Child.child.id).toBe(1);
let calls = 0;
const initialized = [];
function make(C = class {
  static id = calls++;
  static child = calls < 2 ? make() : null;
  static { initialized.push(this.id); }
}) { return C; }
const C = make();
expect(initialized).toEqual([1, 0]);
expect(C.id).toBe(0);
expect(C.child.id).toBe(1);
