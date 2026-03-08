# Ambiguate Properties

- **Status:** Not Implemented
- **Difficulty:** Very Complex

This doc describes related Oxc name-mangling work. It is not part of the new single-file
minifier described in the compressor architecture docs.

## What

Rename unrelated properties on different object shapes to the same short name. If
`Foo.alpha` and `Bar.beta` are never accessed on the same object, both can be renamed to
`.a`. This maximizes property name reuse within a single file, producing shorter output than
renaming each property to a unique short name.

## Why

Design 020 (Mangle Properties) renames all properties to unique short names: `.a`, `.b`,
`.c`, etc. Ambiguation goes further by allowing _different_ original names to share the
_same_ mangled name, as long as they belong to disjoint object shapes. This dramatically
increases name reuse in files with many distinct object shapes:

```js
// Before
class Dog {
  bark() {}
  fetch() {}
}
class Cat {
  meow() {}
  purr() {}
}

// After mangle properties (020) — unique names
class Dog {
  a() {}
  b() {}
}
class Cat {
  c() {}
  d() {}
}

// After ambiguate properties (039) — reused names
class Dog {
  a() {}
  b() {}
}
class Cat {
  a() {}
  b() {}
} // reuses a, b because Dog and Cat are disjoint
```

## How It Works

1. **Build a type graph** — infer which properties belong to which object types. Each object literal, class, or constructor creates a type. Property assignments and accesses associate property names with types.

2. **Build a property interference graph** — two property names interfere if they are ever accessed on the same type (or types connected by prototype chains, unions, or type narrowing). Properties that never co-occur on the same type are non-interfering.

3. **Graph color the property names** — assign short names (colors) to properties such that interfering properties get different names. Non-interfering properties can share the same short name. Use greedy coloring sorted by frequency (most-used properties get shortest names).

### Safety constraints

- **Requires type information** — without type annotations or richer shape inference, the
  analysis must be conservative. Any property accessed on an unknown-typed object must be
  assumed to interfere with all other properties of the same name
- **Prototype chains** — properties inherited through `__proto__` must be treated as belonging to all types in the chain
- **Dynamic access** — `obj[expr]` with non-constant keys disables ambiguation for that object's type
- **`Object.keys()`/`for...in`** — if property names are observed as runtime strings, renaming (and especially reuse) can change behavior
- **Externs** — properties on browser APIs, Node.js APIs, and other external interfaces must not be renamed
- **Quoted properties** — `obj["foo"]` preserves the name by convention (Closure Compiler's rule)

### Difference from design 020

Design 020 (Mangle Properties) assigns a unique short name to each original property name. Ambiguation reuses short names across unrelated properties. Ambiguation is strictly more aggressive and requires type analysis that mangling does not.

## References

- Closure Compiler: `AmbiguateProperties.java` — uses type system to build interference graph and graph-color property names
- Closure Compiler: `DisambiguateProperties.java` — the inverse: renames same-named properties on different types to _different_ names (enabling per-type dead property removal)
- Design 020 (Mangle Properties) — simpler property renaming without type-based reuse
- esbuild: does not implement property ambiguation
- Terser: does not implement property ambiguation
