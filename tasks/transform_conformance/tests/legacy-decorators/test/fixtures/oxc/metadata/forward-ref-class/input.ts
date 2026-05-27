// `LaterClass` resolves to a same-file class declared after the decorated class.
// The guard `typeof X === "undefined" ? Object : X` throws `ReferenceError` at
// decoration time (TDZ), matching SWC and babel.

declare function dec(target: any, key: string, descriptor: PropertyDescriptor): void;

class Source {
  @dec laterRef!: LaterClass;
}

class LaterClass {
  tag = 'later';
}
