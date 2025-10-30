# Minifier Assumptions

The Oxc minifier makes certain assumptions about JavaScript code to achieve optimal compression. These assumptions are standard for typical JavaScript but may not hold for unusual code patterns.

These assumptions are validated using ECMAScript operations from [`oxc_ecmascript`](../oxc_ecmascript), which implements spec-compliant behavior for type conversions, side effect analysis, and constant evaluation.

## Core Assumptions

These assumptions are held regardless of the options.

### No Monkey-Patching Built-ins

[Built-in objects and their methods and properties](https://tc39.es/ecma262/multipage/global-object.html#sec-global-object) behave as specified in ECMAScript.

```javascript
// The minifier assumes this never happens:
Array.prototype.push = function() {
  console.log('hijacked!');
};
Object.defineProperty(Number.prototype, 'toString', { value: () => 'hijacked!' });
```

### No `document.all` Usage

The deprecated [`document.all`](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-IsHTMLDDA-internal-slot) with its special typeof behavior is not used.

```javascript
// The minifier assumes this never happens:
typeof document.all === 'undefined'; // true in browsers
document.all && console.log('exists but falsy');
```

### No `with` Statement

Code does not use `with` statements which create ambiguous scope.

```
// The minifier assumes this never happens:
with (obj) {
  x = 1; // Is this obj.x or a global x?
}
```

### Coercion Methods Are Pure

`.toString()`, `.valueOf()`, and `[Symbol.toPrimitive]()` have no side effects. The minifier uses `oxc_ecmascript` type conversion utilities that assume standard coercion behavior.

```javascript
// The minifier assumes this never happens:
const obj = {
  toString() {
    console.log('side effect!');
    return '';
  },
};
String(obj); // Would trigger side effect
```

### No TDZ Violation

Code doesn't violate Temporal Dead Zones.

```javascript
// The minifier assumes this never happens:
console.log(x); // TDZ violation
let x = 1;
```

### No errors from Array/String Maximum Length

Creating strings or arrays that exceed maximum length can be moved or removed.

```javascript
// The minifier may change when this error occurs:
try {
  new Array(2 ** 32); // RangeError
} catch {
  console.log('caught');
}
```

### No side effects from extending a class

Extending a class does not have a side effect.

```javascript
const v = [];
class A extends v {} // TypeError
```

### Variables declared in direct `eval` are not referenced outside the eval

Variables declared in direct `eval` are not referenced outside the eval, which is only allowed in non-strict mode.

```javascript
// The minifier assumes this never happens:
eval('var x = 1');
console.log(x); // 1
```

### No side effects from accessing to a global variable named `arguments`

Accessing a global variable named `arguments` does not have a side effect. We intend to change this assumption to optional in the future.

```javascript
// The minifier assumes this never happens:
console.log(arguments); // ReferenceError: arguments is not defined
```

## Optional Assumptions

### No Reliance on Function.prototype.name

Code doesn't depend on function names being preserved. This assumption is held by default. This can be changed by settings `keepNames` option.

```javascript
// The minifier assumes this never happens:
function myFunc() {}
if (myFunc.name !== 'myFunc') throw Error();
```

## Configuration

Optional assumptions can be configured in the minifier options if your code requires different behavior.

```rust
pub struct CompressOptions {
    // Control optimization behavior
    pub drop_console: bool,
    pub drop_debugger: bool,
    pub join_vars: bool,
    pub sequences: bool,
    pub unused: CompressOptionsUnused,
    pub keep_names: CompressOptionsKeepNames,

    // Tree-shaking options affect side effect analysis
    pub treeshake: TreeShakeOptions,
}

pub struct TreeShakeOptions {
    // Whether property reads have side effects
    pub property_read_side_effects: PropertyReadSideEffects,
    // Whether accessing unknown globals has side effects
    pub unknown_global_side_effects: bool,
    // Respect pure annotations like /* @__PURE__ */
    pub annotations: bool,
}
```

## Validation

To ensure your code works with these assumptions:

1. **Test thoroughly**: Run your test suite on minified code
2. **Use conformance tests**: `cargo coverage` runs test262, Babel, TypeScript tests
3. **Node.js compatibility**: The minifier is validated against Node.js compatibility tables
4. **Try real-world code**: Test with your actual application
5. **Report issues**: If valid code breaks, file an issue

### Node.js Compatibility Testing

The minifier includes comprehensive validation against Node.js compatibility tables to ensure optimizations work correctly across different Node.js versions and environments.

## When Assumptions Don't Hold

If your code violates these assumptions, you may:

1. Refactor code to follow standard patterns
2. Disable specific optimizations
3. Use a different minifier configuration
4. Report the issue for consideration
