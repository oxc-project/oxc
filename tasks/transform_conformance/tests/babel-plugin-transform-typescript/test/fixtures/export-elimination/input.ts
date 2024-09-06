import Im, {Ok} from 'a';
class Foo {}
const Bar = 0;
function Func() {}
type Baz = any;
interface Baq {}
namespace Name {
  export const Q = 0;
}

export { Im, Ok, Foo, Bar, Func, Baz, Baq, Name };

type T = number;
function T(): T {
  return 123;
}
export { T }
