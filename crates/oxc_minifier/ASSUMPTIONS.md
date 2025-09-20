# Minifier Assumptions

The Oxc minifier makes certain assumptions about JavaScript code to achieve optimal compression. These assumptions are standard for typical JavaScript but may not hold for unusual code patterns.

These assumptions are validated using ECMAScript operations from [`oxc_ecmascript`](../oxc_ecmascript), which implements spec-compliant behavior for type conversions, side effect analysis, and constant evaluation.

## Core Assumptions

### 1. No Monkey-Patching Built-ins

Built-in objects and their methods behave as specified in ECMAScript.

```javascript
// The minifier assumes this never happens:
Array.prototype.push = function() {
  console.log('hijacked!');
};
Object.defineProperty(Number.prototype, 'toString', { value: () => 'hijacked!' });
```

### 2. No `document.all` Usage

The deprecated `document.all` with its special typeof behavior is not used.

```javascript
// The minifier assumes this never happens:
typeof document.all === 'undefined'; // true in browsers
document.all && console.log('exists but falsy');
```

### 3. No `with` Statement

Code does not use `with` statements which create ambiguous scope.

```
// The minifier assumes this never happens:
with (obj) {
  x = 1; // Is this obj.x or a global x?
}
```

### 4. No Direct `eval` or `Function` Constructor

Code doesn't dynamically evaluate strings as code.

```javascript
// The minifier assumes this never happens:
eval('var x = 1');
new Function('return x');
```

### 5. No Arguments Aliasing

The `arguments` object is not aliased or modified in ways that affect parameters.

```javascript
// The minifier assumes this never happens:
function f(a) {
  arguments[0] = 2;
  return a; // Would be affected by arguments modification
}
```

### 6. Getters/Setters Are Pure

Property getters and setters have no side effects.

```javascript
// The minifier assumes this never happens:
const obj = {
  get prop() {
    console.log('side effect!');
    return 1;
  },
};
```

### 7. Coercion Methods Are Pure

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

### 8. No Reliance on Function.prototype.name

Code doesn't depend on function names being preserved.

```javascript
// The minifier assumes this never happens:
function myFunc() {}
if (myFunc.name !== 'myFunc') throw Error();
```

### 9. No Reliance on Function.length

Code doesn't depend on function arity.

```javascript
// The minifier assumes this never happens:
function f(a, b, c) {}
if (f.length !== 3) throw Error();
```

### 10. Regular Prototype Chains

Objects have standard prototype chains without modifications.

```javascript
// The minifier assumes this never happens:
Object.setPrototypeOf(obj, null);
obj.__proto__ = weird_proto;
```

### 11. Special Handling of `__proto__` Property

The minifier correctly handles `__proto__` as a special property name when inlining variables.

```javascript
// Before optimization:
function wrapper() {
  var __proto__ = [];
  return { __proto__ }
}

// After optimization:
function wrapper() {
  return { ['__proto__']: [] }
}
```

The minifier converts shorthand `__proto__` properties to computed form to preserve semantics.

### 12. No TDZ Violation

Code doesn't violate Temporal Dead Zones.

```javascript
// The minifier assumes this never happens:
console.log(x); // TDZ violation
let x = 1;
```

### 13. typeof-Guarded Global Access

The minifier optimizes `typeof`-guarded global variable access by removing unnecessary checks.

```javascript
// Before optimization:
typeof x !== 'undefined' && x

// After optimization:
// (removed entirely if x is known to be undefined)
```

This assumes that `typeof`-guarded expressions are used defensively and can be safely removed when the variable is provably undefined.

### 14. Errors from Array/String Maximum Length

Creating strings or arrays that exceed maximum length can be moved or removed.

```javascript
// The minifier may change when this error occurs:
try {
  new Array(2 ** 32); // RangeError
} catch {
  console.log('caught');
}
```

## Configuration

These assumptions can be configured in the minifier options if your code requires different behavior.

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
