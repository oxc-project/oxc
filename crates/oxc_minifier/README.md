# Minifier

A JavaScript minifier has three components:

1. printer
2. mangler
3. compressor

## Mangler

The mangler implementation is part of the `SymbolTable` residing in `oxc_semantic`.
It is responsible for shortening variables. Its algorithm should be gzip friendly.

The printer is also responsible for printing out the shortened variable names.

## Compressor

The compressor is responsible for rewriting statements and expressions for minimal text output.
[Terser](https://github.com/terser/terser) is a good place to start for learning the fundamentals.

### Assumptions

- [Properties of the global object defined in the ECMAScript spec](https://tc39.es/ecma262/multipage/global-object.html#sec-global-object) behaves the same as in the spec
  - Examples of properties: `Infinity`, `parseInt`, `Object`, `Promise.resolve`
  - Examples that breaks this assumption: `globalThis.Object = class MyObject {}`
- [`document.all`](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-IsHTMLDDA-internal-slot) is not used or behaves as a normal object
  - Examples that breaks this assumption: `console.log(typeof document.all === 'undefined')`
- TDZ violation does not happen
  - Examples that breaks this assumption: `(() => { console.log(v); let v; })()`
- `with` statement is not used
  - Examples that breaks this assumption: `with (Math) { console.log(PI); }`
- Errors thrown when creating a String or an Array that exceeds the maximum length can disappear or moved
  - Examples that breaks this assumption: `try { new Array(Number(2n**53n)) } catch { console.log('log') }`

## Terser Tests

The fixtures are copied from https://github.com/terser/terser/tree/v5.9.0/test/compress
