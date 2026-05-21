// Namespace-imported enum (qualified `NS.E`): the guard chains
// `typeof NS === "undefined" || typeof NS.E === "undefined"`, short-circuiting
// safely if the namespace itself is missing — matching SWC.

import * as NS from './enums';

declare function dec(target: any, key: string): void;

class Source {
  @dec value!: NS.StringEnum;
}
