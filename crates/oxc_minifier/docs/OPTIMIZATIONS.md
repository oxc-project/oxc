# Optimization Catalog

Complete list of optimizations for maximum size reduction.

Many optimizations rely on [`oxc_ecmascript`](../oxc_ecmascript) for ECMAScript operations like constant evaluation, type conversion, and side effect analysis.

## Current Optimizations (18)

### Constant Folding

**Module**: `fold_constants.rs`
**Size Impact**: High
**Description**: Evaluates expressions at compile time using `oxc_ecmascript::constant_evaluation`

```javascript
// Before
2 + 3;
'a' + 'b';

// After
5;
'ab';
```

### Dead Code Elimination

**Module**: `remove_dead_code.rs`
**Size Impact**: Very High
**Description**: Removes unreachable code

```javascript
// Before
if (false) {
  console.log('never runs');
}

// After
// (removed entirely)
```

### Control Flow Optimization

**Module**: `minimize_conditions.rs`, `minimize_if_statement.rs`
**Size Impact**: Medium-High
**Description**: Simplifies conditional logic and if statements

```javascript
// Before
if (x) return true;
else return false;

// After
return !!x;

// Before
if (a) b();

// After
a && b();
```

### Expression Simplification

**Module**: `minimize_logical_expression.rs`, `minimize_not_expression.rs`
**Size Impact**: Medium
**Description**: Simplifies boolean, logical, and negation expressions

```javascript
// Before
!(!x || !y);

// After
x && y;

// Before
!!true;

// After
true;
```

### Syntax Substitution

**Module**: `substitute_alternate_syntax.rs`
**Size Impact**: High
**Description**: Uses shorter equivalent syntax leveraging `oxc_ecmascript` type conversions

```javascript
// Before
true;
false;
undefined;

// After
!0;
!1;
void 0;
```

### Property Access Optimization

**Module**: `convert_to_dotted_properties.rs`
**Size Impact**: Medium
**Description**: Converts bracket notation to dot notation when safe

```javascript
// Before
obj['property'];

// After
obj.property;
```

### Template Literal Optimization

**Module**: `inline.rs`
**Size Impact**: Medium
**Description**: Simplifies template literals

```javascript
// Before
`hello ${'world'}`;

// After
'hello world';
```

### Built-in Method Replacement

**Module**: `replace_known_methods.rs`
**Size Impact**: High
**Description**: Optimizes known built-in method calls

```javascript
// Before
'test'.indexOf('e');
Math.pow(2, 3);

// After
'test'.indexOf('e'); // or optimized form
2 ** 3;
```

### For Statement Optimization

**Module**: `minimize_for_statement.rs`
**Size Impact**: Low-Medium
**Description**: Optimizes for loops

```javascript
// Before
for (;;) {}

// After
for (;;);
```

### Statement-Level Optimizations

**Module**: `minimize_statements.rs`
**Size Impact**: Medium
**Description**: Combines and simplifies statements

```javascript
// Before
a();
b();

// After (with sequences option)
a(), b();
```

### Unused Code Removal

**Module**: `remove_unused_declaration.rs`, `remove_unused_expression.rs`
**Size Impact**: High
**Description**: Removes unused variables and simplifies unused expressions

```javascript
// Before
let unused = 5;
console.log('hello');

// After
console.log('hello');

// Before
[1, 2, fn()]; // unused array with side effects

// After
fn(); // keep only side effects
```

### Normalization

**Module**: `normalize.rs`
**Size Impact**: Enables other optimizations
**Description**: Converts code to canonical form

```javascript
// Example: while -> for conversion
// Before
while (true) {}

// After
for (;;) {}
```

### Conditional Expression Optimization

**Module**: `minimize_conditional_expression.rs`
**Size Impact**: Medium
**Description**: Optimizes ternary operators

```javascript
// Before
x ? true : false;

// After
!!x;
```

### Boolean Context Optimization

**Module**: `minimize_expression_in_boolean_context.rs`
**Size Impact**: Low-Medium
**Description**: Simplifies expressions in boolean contexts

```javascript
// Before
if (x === true) {}

// After
if (x) {}
```

### Variable Inlining

**Module**: `inline.rs`
**Size Impact**: Medium
**Description**: Inlines single-use variables

```javascript
// Before
const x = 5;
console.log(x);

// After
console.log(5);
```

## Planned Optimizations

### From Closure Compiler

- **Cross-module constant propagation**: Track constants across files
- **Advanced function inlining**: Inline functions when beneficial
- **Enum unboxing**: Replace enum objects with primitives
- **Property collapsing**: Flatten nested property access
- **Method devirtualization**: Direct method calls when possible

### From Terser

- **Switch statement optimization**: Simplify switch statements
- **Advanced array/object patterns**: Recognize and optimize patterns
- **String optimizations**: Join strings, optimize concatenation
- **RegExp optimizations**: Simplify regular expressions

### From esbuild

- **CSS-in-JS optimization**: Optimize styled components
- **Import optimization**: Remove unused imports
- **Enum inlining**: Replace enum access with values

## Size Optimization Techniques

1. **Multiple passes**: Run optimizations until fixed-point
2. **Optimization ordering**: Some optimizations enable others
3. **Pattern matching**: Recognize common code patterns
4. **Syntactic substitution**: Always prefer shorter forms
5. **Exhaustive application**: Apply everywhere possible

## How Optimizations Interact

Some optimizations enable others:

- **Normalization** → enables better pattern matching
- **Constant folding** → enables dead code elimination
- **Inlining** → enables further constant folding
- **Dead code removal** → enables more inlining

The fixed-point iteration ensures all optimization opportunities are found.
