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
console.log(JSON.stringify(seen), result.Child.id);
