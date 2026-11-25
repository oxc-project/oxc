let depth = 0;
const seen = [];
class Outer {
  Child = (() => {
    var _staticBlock;
    return { "Child": class {
      static id = depth++;
      static child = ((_value) => (_staticBlock = () => (seen.push(this.id), this), _value))(depth < 2 ? new Outer().Child : null);
    } }["Child"], _staticBlock();
  })();
}
const result = new Outer();
console.log(JSON.stringify(seen), result.Child.id);
