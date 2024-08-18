import 'foo';
declare module 'foo' {
    interface Foo {}
    const foo = 42;
}

declare global {
    interface Bar {}
    const bar = 42 ;
}

// should not be emitted
module baz {
    interface Baz {}
    const baz = 42;
}

declare module x {
    interface Qux {}
    const qux = 42;
}
