enum Foo { x = 1 }
enum Bar { y = 2 }

// `Foo?.x` followed by another property access — the inner `?.x` reference
// must still be inlined and the enum removed.
console.log(Foo?.x.toString());
console.log(Bar?.['y']?.toString());
