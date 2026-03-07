# Peephole Collect Property Assignments

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

This pass merges sequences of property assignments that immediately follow an object or array literal declaration into the literal itself. The separate assignment statements are removed and their values are folded into the initializer.

## Why

A common coding pattern is to declare an empty object or array and then populate it with assignments on subsequent lines. Each assignment statement has syntactic overhead (the variable name, brackets/dot, semicolon). By collecting these assignments back into the literal, the code is shorter and eliminates repeated references to the variable name.

## Transformations

### Object property collection

Merge consecutive property assignments into the object literal.

```js
// Before
var o = {};
o.x = 1;
o.y = 2;
o.z = 3;

// After
var o = {x: 1, y: 2, z: 3};
```

### Array element collection

Merge consecutive indexed assignments into the array literal.

```js
// Before
var a = [];
a[0] = "x";
a[1] = "y";
a[2] = "z";

// After
var a = ["x", "y", "z"];
```

### Extend existing literals

Append to non-empty literals when assignments follow immediately.

```js
// Before
var o = {a: 1};
o.b = 2;
o.c = 3;

// After
var o = {a: 1, b: 2, c: 3};
```

### Sparse arrays

Handle non-contiguous array indices by inserting holes.

```js
// Before
var a = [];
a[0] = "first";
a[3] = "fourth";

// After
var a = ["first",,, "fourth"];
```

### Stop at non-trivial assignments

Stop collecting when an assignment's right-hand side references the target variable or when a non-assignment statement intervenes.

```js
// Before
var a = [];
a[0] = 1;
console.log("break");  // intervening statement
a[1] = 2;

// After
var a = [1];
console.log("break");
a[1] = 2;
```

```js
// Before
var o = {};
o.self = o;  // RHS references o — stop here

// After (no change, RHS depends on o)
var o = {};
o.self = o;
```

## References

- `PeepholeCollectPropertyAssignments.java`
- `js_ast_helpers.go:2737,2716` (spread inlining only)
- `compress/tighten-body.js:1370` (`join_object_assignments`)
