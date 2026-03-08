# Hoist Properties

- **Status:** Not Implemented
- **Difficulty:** Simple-Medium

## What

Extract constant properties from object literals into standalone variables. When an object is only used for property access (never escapes as a whole), each property can be lifted to a separate variable, eliminating the object entirely. After mangling, the individual variables compress better than object property access chains.

## Why

Developers frequently group related constants or configuration values into objects. If these objects are only accessed by known property names, the object wrapper is overhead — property access (`o.x`) is longer than a direct variable reference (`a`), and the object literal syntax (`{x:1,y:2}`) is longer than separate declarations (`var a=1,b=2`). This pass converts structured access patterns into flat variable references that mangle and compress more effectively.

## Conceptual Dependencies

This design depends primarily on **alias and escape reasoning**. Property hoisting is only
correct when the object can still be treated as a local container instead of as shared
identity. The optimizer therefore needs a way to distinguish "property reads on a local
record-like object" from "observable object state that may be inspected elsewhere."

## Transformations

### Basic property hoisting

```js
// Before
var config = { width: 100, height: 200 };
draw(config.width, config.height);

// After
var config_width = 100,
  config_height = 200;
draw(config_width, config_height);
```

### Nested object property access

```js
// Before
var o = { a: 1, b: [2, 3] };
f(o.a);

// After (only hoist primitive properties)
var o_a = 1;
var o = { b: [2, 3] };
f(o_a);
```

### After mangling

The real benefit appears after variable name mangling:

```js
// Before mangling
var config_width = 100,
  config_height = 200;
draw(config_width, config_height);

// After mangling
var a = 100,
  b = 200;
draw(a, b);
```

### Safety constraints

- The object must not escape — cannot be passed as an argument, returned, assigned to another variable, or used with spread/rest
- Only hoist properties with constant (literal) values — not function calls or mutable references
- No computed property access (`o[expr]`) — all accesses must be static dot notation or string literal bracket notation
- No `Object.keys()`, `Object.values()`, `for...in`, or other whole-object operations
- The object variable must have a single assignment (declared with initializer, never reassigned)
- Properties must not be reassigned after initialization

## References

- Terser: `hoist_props: true` — hoists properties from constant object/array literals to standalone variables
- SWC: `hoist_props` — same behavior as Terser
- Closure: `InlineProperties.java` — related pass that inlines object properties at use sites
- esbuild: Does not implement property hoisting
