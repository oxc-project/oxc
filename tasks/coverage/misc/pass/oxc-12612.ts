// Type assertions with valid assignment targets should parse successfully
// https://github.com/oxc-project/oxc/issues/12612

let a = 1;
let obj = { foo: 1 };

// Simple type assertions with identifiers
(a as any) = 42;
(<any>a) = 42;
(a satisfies number) = 42;

// Type assertions with member expressions
(obj.foo as any) = 42;
(<any>obj.foo) = 42;
(obj.foo satisfies number) = 42;

// Nested type assertions should work when the inner expression is assignable
(a as number as any) = 42;
(a satisfies number as any) = 42;
(<any>(a as number)) = 42;
(<number>(<any>a)) = 42;

// Mixed type assertions
(a! as any) = 42;
((a as any)!) = 42;
