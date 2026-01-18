// Test namespace declarations
export namespace Foo {
    export var a = 2;
    export function bar(): void {}
}

// Nested namespaces
export namespace Outer {
    export namespace Inner {
        export var x = 1;
    }
}

// Should preserve namespace as-is
export namespace Baz {
    export var c = 3;
}

// Should preserve global as-is
declare global {
    interface GlobalTest {}
}

// Should preserve string modules as-is
declare module "test-module" {
    export var d = 4;
}