// Type-only export imported without the `type` keyword: `T` is `undefined` at
// runtime after erasure. The guard falls back to `Object` rather than
// `undefined`, matching SWC and babel.

import { T } from './m';

declare function dec(target: any, key: string): void;

class Source {
  @dec x!: T;
}
