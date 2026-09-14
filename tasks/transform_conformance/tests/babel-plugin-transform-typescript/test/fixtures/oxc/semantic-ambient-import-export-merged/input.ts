import type { Foo } from './foo';
import { type Bar } from './bar';
import type { Live } from './live';
declare const Foo: Foo;
declare const Bar: Bar;
const Live = 1;
export { Foo, Live };
export default Bar;
