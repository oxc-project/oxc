// Forward-referenced class: the type annotation `LaterClass` resolves to a
// same-file class declaration that comes after the decorated class. Previously
// OXC wrapped this in a `typeof X !== "undefined"` guard which silently emitted
// `Object` (because the binding was hoisted but unset at decoration time).
// The patched emit produces a bare identifier, matching tsc and babel; the
// JS runtime then surfaces the ordering bug as `ReferenceError: Cannot access
// 'LaterClass' before initialization`.

declare function dec(target: any, key: string, descriptor: PropertyDescriptor): void;

class Source {
  @dec laterRef!: LaterClass;
}

class LaterClass {
  tag = 'later';
}
