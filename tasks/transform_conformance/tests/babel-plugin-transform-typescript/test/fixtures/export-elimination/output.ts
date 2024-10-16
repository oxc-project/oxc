import Im, { Ok } from "a";
class Foo { }
const Bar = 0;
function Func() { }
let Name;
(function (_Name) {
  const Q = _Name.Q = 0;
})(Name || (Name = {}));

export { Im, Ok, Foo, Bar, Func, Name };

function T() {
  return 123;
}
export { T }
