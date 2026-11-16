use oxc_transformer::TransformOptions;

use crate::test;

#[test]
fn object_spread_proto_after_spread() {
    let options = TransformOptions::from_target("es2017").unwrap();

    let check = |code: &str| test(code, &options).unwrap();

    // A non-computed `__proto__` after a spread must set the prototype of the
    // resulting object, not the temporary helper argument.
    let out = check("const o = { ...a, __proto__: base };");
    assert!(out.contains("Object.setPrototypeOf"));
    assert!(out.contains("typeof _proto === 'object' || typeof _proto === 'function'"));

    // String-literal `__proto__` has the same prototype-setter semantics.
    let out = check("const o = { ...a, '__proto__': base };");
    assert!(out.contains("Object.setPrototypeOf"));
    assert!(out.contains("typeof _proto === 'object' || typeof _proto === 'function'"));

    // When `__proto__` precedes the first spread it stays on the initial target
    // object and works without any extra wrapper.
    let out = check("const o = { __proto__: base, ...a };");
    assert!(!out.contains("Object.setPrototypeOf"));
    assert!(out.contains("{ __proto__: base }"));

    // Computed `['__proto__']` is an ordinary data property, not a prototype setter.
    let out = check("const o = { ...a, ['__proto__']: base };");
    assert!(!out.contains("Object.setPrototypeOf"));
}
