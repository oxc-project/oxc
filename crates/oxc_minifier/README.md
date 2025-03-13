# Minifier

A JavaScript minifier has three components:

1. compressor
2. mangler
3. printer

## Compressor

The compressor is responsible for rewriting statements and expressions for minimal text output.
[Terser](https://github.com/terser/terser) is a good place to start for learning the fundamentals.

## Mangler

The mangler implementation is part of the `SymbolTable` residing in `oxc_semantic`.
It is responsible for shortening variables. Its algorithm should be gzip friendly.

The printer is also responsible for printing out the shortened variable names.

## Printer

The printer is responsible for removing whitespace from the source text.

### Assumptions

- [Properties of the global object defined in the ECMAScript spec](https://tc39.es/ecma262/multipage/global-object.html#sec-global-object) behaves the same as in the spec
  - Examples of properties: `Infinity`, `parseInt`, `Object`, `Promise.resolve`
  - Examples that breaks this assumption: `globalThis.Object = class MyObject {}`
- The code does not rely on the `name` property of `Function` or `Class`
  - Examples that breaks this assumption: `function fn() {}; console.log(f.name === 'fn')`
- [`document.all`](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-IsHTMLDDA-internal-slot) is not used or behaves as a normal object
  - Examples that breaks this assumption: `console.log(typeof document.all === 'undefined')`
- TDZ violation does not happen
  - Examples that breaks this assumption: `(() => { console.log(v); let v; })()`
- `with` statement is not used
  - Examples that breaks this assumption: `with (Math) { console.log(PI); }`
- `.toString()`, `.valueOf()`, `[Symbol.toPrimitive]()` are side-effect free
  - Examples that breaks this assumption: `{ toString() { console.log('sideeffect') } }`
- Errors thrown when creating a String or an Array that exceeds the maximum length can disappear or moved
  - Examples that breaks this assumption: `try { new Array(Number(2n**53n)) } catch { console.log('log') }`
- Invalid super class error does not happen
  - Examples that breaks this assumption: `const v = []; class A extends v {}`

## Terser Tests

The fixtures are copied from https://github.com/terser/terser/tree/v5.9.0/test/compress
