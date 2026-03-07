# Mangle Properties

- **Status:** Not Implemented
- **Difficulty:** Medium

## What

This pass renames object property names to shorter identifiers, consistently replacing every occurrence of a given property name throughout the program. It is the property-level analog of variable name mangling (scope-based renaming).

## Why

Property names are often long and descriptive (`backgroundColor`, `handleClickEvent`, `isAuthenticated`). Unlike variable names — which are scoped and can be shortened independently per scope — property names are global: every access to `.backgroundColor` across the program must use the same name. Shortening them yields substantial savings in property-heavy code, but requires careful consistency.

## How It Works

1. **Collect** — scan the entire program for property names used in dot access, bracket access (with string literals), object literals, class definitions, destructuring patterns, and `Object.defineProperty` calls
2. **Filter** — exclude property names that must not be renamed:
   - DOM API properties (`innerHTML`, `addEventListener`, etc.)
   - Well-known symbols and protocol properties (`toString`, `valueOf`, `constructor`, `prototype`)
   - Properties accessed via dynamic bracket notation with non-constant keys
   - Properties listed in a user-provided "reserved" list
3. **Sort** — order property names by frequency (most-used first)
4. **Assign** — assign short names from a name generator (`a`, `b`, ..., `z`, `A`, ..., `aa`, ...) to properties in frequency order, so the most common properties get the shortest names
5. **Rename** — walk the AST and replace all occurrences of each property name with its assigned short name

## Transformations

### Basic property mangling

```js
// Before
var obj = {
  backgroundColor: "red",
  fontSize: 14,
  handleClick: function () {},
};
obj.backgroundColor = "blue";
obj.handleClick();

// After
var obj = {
  a: "red",
  b: 14,
  c: function () {},
};
obj.a = "blue";
obj.c();
```

### Class properties

```js
// Before
class Widget {
  constructor() {
    this.isVisible = true;
    this.elementCount = 0;
  }
  toggleVisibility() {
    this.isVisible = !this.isVisible;
  }
}

// After
class Widget {
  constructor() {
    this.a = true;
    this.b = 0;
  }
  c() {
    this.a = !this.a;
  }
}
```

### Destructuring patterns

Property names in destructuring must be renamed consistently.

```js
// Before
var { backgroundColor, fontSize } = getStyles();
use(backgroundColor);

// After
var { a: backgroundColor, b: fontSize } = getStyles();
use(backgroundColor);
```

Note: the destructured _binding_ names (local variables) are handled by variable mangling, not property mangling.

### Quoted properties

By default, quoted property names (`obj["foo"]`) are eligible for mangling. A `--mangle-quoted` option can control whether quoted properties are included or excluded.

```js
// With quote mangling enabled:
obj["longPropertyName"]  →  obj["a"]  →  obj.a  // (after convert-to-dotted)
```

### Consistency across files

In a multi-file bundle, the same property name must always map to the same short name. The property renaming map can be serialized and reused across builds to ensure stability.

### Reserved properties

Users can specify property names to exclude from mangling:

```js
// Config: reserve ["onClick", "render"]
var c = {
  onClick: handler, // preserved
  render: function () {}, // preserved
  internalState: {}, // mangled to "a"
};
```

## Risks

Property mangling is the most **dangerous** minification optimization because:

- External code (libraries, APIs, JSON) must use the same property names at runtime
- Dynamic property access (`obj[variable]`) cannot be statically analyzed
- Serialization (`JSON.stringify`) exposes mangled names

It is typically opt-in and requires the developer to explicitly configure reserved names or use a naming convention (e.g., mangle only properties matching a pattern like `_prefix`).

## References

- `RenameProperties.java`
- `linker/linker.go:352`, `js_parser.go:2846`
- `propmangle.js:216`
