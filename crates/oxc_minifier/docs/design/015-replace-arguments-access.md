# Replace Arguments Access

- **Status:** Not Implemented
- **Difficulty:** Simple

## What

Replace `arguments` object access with named parameters. When a function uses `arguments[N]` to access positional arguments, replace with the corresponding parameter name.

## Why

The `arguments` object is expensive — engines must create the object and maintain a live binding with named parameters. Replacing `arguments[N]` with a named parameter eliminates the `arguments` object entirely (if no other references remain), enabling better engine optimization and smaller output.

## Transformations

### Replace indexed access with parameter name

```js
// Before
function f(a, b) {
  return arguments[0] + arguments[1];
}

// After
function f(a, b) {
  return a + b;
}
```

### Add missing parameters

When `arguments[N]` exceeds the parameter count, add synthetic parameters to enable replacement.

```js
// Before
function f() {
  return arguments[0] + arguments[1];
}

// After
function f(argument_0, argument_1) {
  return argument_0 + argument_1;
}
```

### Replace `arguments.length`

When the function has no rest parameters and arguments are not modified, replace with the parameter count.

```js
// Before
function f(a, b) {
  return arguments.length;
}

// After
function f(a, b) {
  return 2;
}
```

Note: This is only safe for functions that are always called with exactly the declared number of arguments (requires call-site analysis or must be guarded).

### Safety constraints

Do NOT replace when:

- `arguments` is aliased (`var args = arguments`)
- `arguments` is passed to another function (`f(arguments)`)
- `arguments.callee` is used
- `arguments` is used with spread/apply
- A parameter shadows the name `"arguments"`
- A local variable is named `"arguments"`
- The function is an arrow function (no own `arguments`)
- Rest parameters are present (changes `arguments` semantics)
- Destructuring parameters are used (index mapping is complex)

```js
// NOT safe — arguments is aliased
function f(a) {
  var args = arguments;
  return args[0];
}

// NOT safe — arguments escapes
function f(a) {
  g(arguments);
}
```

## References

- Terser: `arguments: false` by default. Checks no parameter/local shadows `"arguments"`, not arrow
- SWC: `optimize_usage_of_arguments` replaces `arguments[N]` with param; `optimize_str_access_to_arguments` converts `arguments['foo']` → `arguments.foo`; injects synthetic `argument_N` params when index exceeds count
- esbuild: Does NOT do this optimization
- Closure: `OptimizeArgumentsArray.java` (advanced mode only). Blocks if `arguments` escapes, rest params present, or destructuring params used. Known bug with arrow functions nested inside target function
