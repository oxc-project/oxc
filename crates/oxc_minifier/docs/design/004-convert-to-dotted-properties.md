# Convert to Dotted Properties

- **Status:** Not Implemented
- **Difficulty:** Trivial

## What

This pass converts bracket notation property access with string literal keys to dot notation, and computed property keys in object/class literals to static keys, when the key is a valid identifier.

## Why

Dot notation (`obj.foo`) is shorter than bracket notation (`obj["foo"]`) by 3 bytes (the two quotes and one bracket, minus the dot). For object literals, `{foo: 1}` is shorter than `{"foo": 1}` or `{["foo"]: 1}`. These savings are small per occurrence but property access is extremely common in JavaScript.

## Transformations

### Bracket access to dot access

Convert `obj["prop"]` to `obj.prop` when the string is a valid identifier name.

```js
// Before
obj["foo"]
obj["bar"]
arr["length"]
this["name"]

// After
obj.foo
obj.bar
arr.length
this.name
```

### Computed property keys to static keys

Convert computed property keys in object literals when the key is a string literal that is a valid identifier.

```js
// Before
var o = {
  ["foo"]: 1,
  ["bar"]: function() {}
};

// After
var o = {
  foo: 1,
  bar: function() {}
};
```

### Class computed properties

Same transformation applies to class method and property definitions.

```js
// Before
class C {
  ["method"]() { return 1; }
  ["prop"] = 2;
}

// After
class C {
  method() { return 1; }
  prop = 2;
}
```

### When NOT to convert

The key must be a valid JavaScript identifier. Do not convert when the string contains reserved words that would change semantics, spaces, hyphens, or starts with a digit.

```js
// These stay as bracket notation:
obj["foo-bar"]      // hyphen is not valid in identifiers
obj["123"]          // starts with digit
obj["class"]        // reserved word (safe in ES5+ property access, but context-dependent)
obj[""]             // empty string
obj["hello world"]  // contains space
```

Note: In ES5+, reserved words *are* valid as property names in dot notation and object literals. So `obj["class"]` → `obj.class` is actually safe in modern JavaScript. The implementation should allow this conversion when targeting ES5+.

### Numeric keys

Convert numeric literal keys to their string equivalent when shorter.

```js
// Before
var o = {0: "a", 1: "b"};

// After (no change — numeric keys are already shorter than quoted)
var o = {0: "a", 1: "b"};
```

## References

- `ConvertToDottedProperties.java`
- `js_parser.go:14391`
- `compress/index.js:3558`
