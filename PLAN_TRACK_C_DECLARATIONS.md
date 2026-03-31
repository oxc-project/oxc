# Track C: Declaration Types (ClassDeclaration, EnumDeclaration, FunctionDeclaration)

## Parallel Work Notice

This track runs IN PARALLEL with:
- **Track A** (expression types: StaticMemberExpression, CallExpression, ArrayExpression) — modifies `expression_type.rs`, `check_expression.rs`
- **Track B** (type node resolution: TSArrayType, TSTupleType, TSFunctionType) — modifies `type_from_type_node.rs`

All three tracks modify different files/methods. The ONLY shared surfaces:
- `declared_type.rs` — **this track owns it**. Tracks A and B should NOT modify this file.
- `checker.rs` — **this track owns `check_source_element`**. Only this track adds new `Statement::*` arms. Track A may add new `Expression::*` arms in `check_expression.rs` (different file, no conflict).
- `type_display.rs` — all tracks may add display arms. Coordinate by adding new `TypeData::*` match arms only (don't touch existing arms).
- `tests.rs` — each track appends tests at the end of the file. No conflicts if appending only.
- `type_data.rs` — already has all needed struct definitions. No modifications needed.

---

## Codebase Overview

### Repository: `/Users/leoshatrushin/dev/oxc` (branch: `oxc-typecheck`)
### Crate: `crates/oxc_checker`

### Architecture
- **TypeArena** (`crates/oxc_types/src/type_arena.rs`): SoA storage with `IndexVec<TypeId, T>` columns for flags, object_flags, data, symbols.
- **TypeId**: u32 newtype index into the arena.
- **TypeData** (`crates/oxc_types/src/type_data.rs`): Enum with 19 variants (Intrinsic, Literal, Object, Interface, TypeReference, Union, Intersection, TypeParameter, Tuple, etc.)
- **TypeFlags** (`crates/oxc_types/src/type_flags.rs`): u32 bitflags classifying type kinds.
- **ObjectFlags** (`crates/oxc_types/src/object_flags.rs`): u32 bitflags. Key flags: `Class`, `Interface`, `Anonymous`, `ObjectLiteral`.

### Key method for creating types:
```rust
type_arena.new_type(flags: TypeFlags, object_flags: ObjectFlags, data: TypeData, symbol: Option<SymbolId>) -> TypeId
```

### How declarations work

**Value-namespace symbols** (variables, functions, classes as values) go through:
- `expression_type.rs: get_type_of_symbol(symbol_id)` → `resolve_symbol_type(symbol_id)`
- Currently handles: `AstKind::VariableDeclarator`, `AstKind::FormalParameter`
- Returns `any_type` for everything else (including function declarations, class declarations)

**Type-namespace symbols** (interfaces, type aliases, classes as types) go through:
- `declared_type.rs: get_declared_type_of_symbol(symbol_id)` → `resolve_declared_type(symbol_id)`
- Currently handles: `AstKind::TSTypeAliasDeclaration`, `AstKind::TSInterfaceDeclaration`
- Returns `any_type` for everything else (including class declarations, enum declarations)
- Has caching via `declared_type_cache: HashMap<SymbolId, TypeId>` and cycle detection via `resolving_symbols: Vec<SymbolId>`

**Statement checking** goes through:
- `checker.rs: check_source_element(stmt)` — dispatches by Statement variant
- Currently handles: VariableDeclaration, FunctionDeclaration, block/control flow, exports, expressions, returns
- Currently no-ops: `ClassDeclaration`, `TSEnumDeclaration` (in the catch-all leaf arm)

### Existing patterns to follow

**Interface declaration** (`declared_type.rs:58-98`):
```rust
fn get_type_of_interface_declaration(&mut self, decl: &TSInterfaceDeclaration) -> TypeId {
    let type_parameters = self.get_type_parameters_from_declaration(decl.type_parameters.as_deref());
    let mut properties = IndexMap::new();
    for sig in &decl.body.body {
        if let TSSignature::TSPropertySignature(prop) = sig {
            if let Some(name) = prop.key.static_name() {
                let prop_type = if let Some(ann) = &prop.type_annotation {
                    self.get_type_from_type_node(&ann.type_annotation)
                } else { self.any_type };
                properties.insert(CompactStr::new(&name), prop_type);
            }
        }
    }
    self.type_arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Interface,
        TypeData::Interface(InterfaceType {
            target: None,
            resolved_type_arguments: SmallVec::new(),
            all_type_parameters: type_parameters,
            this_type: None,
            resolved_base_types: SmallVec::new(),
            properties,
        }),
        None,
    )
}
```

**Variable declarator type resolution** (`expression_type.rs:196-222`):
```rust
fn resolve_symbol_type(&mut self, symbol_id: SymbolId) -> TypeId {
    use oxc_ast::AstKind;
    let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
    let node = self.semantic().nodes().get_node(node_id);
    match node.kind() {
        AstKind::VariableDeclarator(decl) => {
            if let Some(annotation) = &decl.type_annotation {
                self.get_type_from_type_node(&annotation.type_annotation)
            } else if let Some(init) = &decl.init {
                self.get_type_of_expression(init)
            } else {
                self.any_type
            }
        }
        AstKind::FormalParameter(param) => { /* ... */ }
        _ => self.any_type,  // ← ADD NEW ARMS HERE
    }
}
```

### Test pattern
```rust
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement, TSType};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_types::{TypeData, TypeFlags};
use crate::Checker;

macro_rules! with_checker {
    ($source:expr, |$checker:ident, $program:ident| $body:block) => {{
        let allocator = Allocator::default();
        let source_type = SourceType::ts();
        let parsed = Parser::new(&allocator, $source, source_type).parse();
        let $program = &parsed.program;
        let semantic = SemanticBuilder::new().build($program).semantic;
        let mut type_arena = oxc_types::TypeArena::with_capacity(64);
        #[allow(unused_mut)]
        let mut $checker = Checker::new(semantic, &mut type_arena);
        $body
    }};
}
```

### Verification commands
```bash
cd /Users/leoshatrushin/dev/oxc && cargo test -p oxc_checker
cd /Users/leoshatrushin/dev/oxc && cargo run -p oxc_coverage -- checker 2>&1 | tail -10
```

---

## Task 1: ClassDeclaration (HIGHEST PRIORITY — ~93 conformance tests)

### What TypeScript classes look like

```typescript
class Point {
    x: number;
    y: number;
    constructor(x: number, y: number) { this.x = x; this.y = y; }
    distance(): number { return Math.sqrt(this.x * this.x + this.y * this.y); }
}

class Animal {
    name: string;
    readonly legs: number = 4;
}

class Dog extends Animal {
    breed: string;
}
```

### What needs to happen

Classes live in BOTH namespaces:
1. **Type namespace**: `class Foo` creates a type named `Foo` (the instance type)
2. **Value namespace**: `class Foo` creates a value named `Foo` (the constructor)

For v1, focus on the **type namespace** (instance type) since that's what type annotations reference (`let x: Point`).

### 1a. Add ClassDeclaration to `declared_type.rs`

In `resolve_declared_type`, add a new arm:

```rust
AstKind::Class(decl) => {
    self.get_type_of_class_declaration(decl)
}
```

**IMPORTANT**: Check what AstKind variant classes use. It might be `AstKind::Class(class)` or `AstKind::ClassDeclaration(class)`. Find out:
```bash
grep -n "Class\b" crates/oxc_ast/src/generated/ast_kind.rs | head -20
```

New method `get_type_of_class_declaration`:

```rust
fn get_type_of_class_declaration(
    &mut self,
    decl: &oxc_ast::ast::Class<'_>,
) -> TypeId {
    let type_parameters = self.get_type_parameters_from_declaration(
        decl.type_parameters.as_deref(),
    );

    let mut properties = IndexMap::new();

    for element in &decl.body.body {
        use oxc_ast::ast::ClassElement;
        match element {
            ClassElement::PropertyDefinition(prop) => {
                if prop.r#static {
                    continue; // static props go on constructor type, not instance
                }
                if let Some(name) = prop.key.static_name() {
                    let prop_type = if let Some(ann) = &prop.type_annotation {
                        self.get_type_from_type_node(&ann.type_annotation)
                    } else if let Some(init) = &prop.value {
                        self.get_type_of_expression(init)
                    } else {
                        self.any_type
                    };
                    properties.insert(CompactStr::new(&name), prop_type);
                }
            }
            ClassElement::MethodDefinition(method) => {
                // TODO: method signatures — need call signature infrastructure
                // For now, skip methods (they won't appear as properties)
                // Future: store method type as a function type property
            }
            ClassElement::AccessorProperty(accessor) => {
                // TODO: getter/setter types
            }
            ClassElement::TSIndexSignature(_) => {
                // TODO: index signatures
            }
            _ => {}
        }
    }

    // Handle extends clause — resolve base types
    let mut resolved_base_types = SmallVec::new();
    if let Some(super_class) = &decl.super_class {
        let base_type = self.get_type_of_expression(super_class);
        if base_type != self.any_type {
            resolved_base_types.push(base_type);
            // TODO: inherit base class properties into this class's properties
            // For proper inheritance, need to merge base properties
        }
    }

    self.type_arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Class,
        TypeData::Interface(InterfaceType {
            target: None,
            resolved_type_arguments: SmallVec::new(),
            all_type_parameters: type_parameters,
            this_type: None,
            resolved_base_types,
            properties,
        }),
        None,
    )
}
```

**Why use `InterfaceType` for classes?** In tsc/tsgo, class instance types ARE interface types (they share the same data structure). The `ObjectFlags::Class` flag distinguishes them. This matches the existing pattern and means assignability checks work automatically.

**AST exploration needed**: Check the actual oxc AST shape for classes:
```bash
grep -n "pub struct Class\b\|ClassElement\|PropertyDefinition\|MethodDefinition" crates/oxc_ast/src/generated/ast/js.rs | head -30
```

Key things to verify:
- Does `Class` have `type_parameters`? (Yes, for `class Foo<T>`)
- Does `Class` have `super_class`? (Yes, for `class Dog extends Animal`)
- Does `PropertyDefinition` have `type_annotation`? (Yes)
- Does `PropertyDefinition` have `value`? (Yes, for `legs = 4`)
- Does `PropertyDefinition` have `r#static`? (Yes)
- Does `PropertyDefinition.key` have `static_name()`? (Yes, same as other property keys)

### 1b. Add ClassDeclaration to `resolve_symbol_type` in `expression_type.rs`

Classes also need value-namespace resolution (for `let x = new Point()`). For v1:

```rust
AstKind::Class(_) => {
    // The value of a class expression/declaration is the constructor
    // For now, return any — proper class constructor types need
    // construct signatures
    self.any_type
}
```

### 1c. Add ClassDeclaration checking in `checker.rs`

In `check_source_element`, move `Statement::ClassDeclaration(_)` from the catch-all to a real arm:

```rust
Statement::ClassDeclaration(class) => {
    self.check_class_declaration(class);
}
```

New method (can go in `checker.rs` or a new `check_class.rs` file — prefer `checker.rs` for now):

```rust
fn check_class_declaration(&mut self, class: &oxc_ast::ast::Class<'a>) {
    // Check method bodies
    for element in &class.body.body {
        use oxc_ast::ast::ClassElement;
        if let ClassElement::MethodDefinition(method) = element {
            if let Some(body) = &method.value.body {
                let return_type = method.value.return_type
                    .as_ref()
                    .map(|rt| self.get_type_from_type_node(&rt.type_annotation));
                self.return_type_stack.push(return_type);
                self.check_source_elements(&body.statements);
                self.return_type_stack.pop();
            }
        }
    }
    // TODO: check property initializer types against annotations
    // TODO: check that abstract members are implemented in subclasses
}
```

Also update `check_exported_declaration` in `checker.rs` — `Declaration::ClassDeclaration` is currently in the no-op catch-all:

```rust
Declaration::ClassDeclaration(class) => {
    self.check_class_declaration(class);
}
```

### 1d. Display

Classes use `InterfaceType`, which already has a display arm in `type_display.rs` (added in the previous session). The display will show `{ x: number; y: number; }` for a class with those properties. This matches tsc's structural display for class types in `.types` baselines.

If you want named display (showing `Point` instead of the structural form), that requires storing the class name — skip for v1.

### Tests

```rust
#[test]
fn class_declaration_property_types() {
    with_checker!(
        "class Point { x: number; y: string; }",
        |checker, program| {
            // The class declaration itself shouldn't produce errors
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn class_as_type_annotation() {
    with_checker!(
        "class Point { x: number; y: number; } let p: Point = { x: 1, y: 2 }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn class_type_missing_property() {
    with_checker!(
        "class Point { x: number; y: number; } let p: Point = { x: 1 }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(!diagnostics.is_empty(), "should error: missing property y");
        }
    );
}

#[test]
fn class_type_wrong_property_type() {
    with_checker!(
        "class Point { x: number; y: number; } let p: Point = { x: 1, y: 'hello' }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(!diagnostics.is_empty(), "should error: string not assignable to number");
        }
    );
}

#[test]
fn class_with_initializer() {
    with_checker!(
        "class Config { debug: boolean; name: string; }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}
```

---

## Task 2: EnumDeclaration

### What TypeScript enums look like

```typescript
enum Direction { Up, Down, Left, Right }       // numeric enum
enum Color { Red = 1, Green = 2, Blue = 4 }    // numeric with values
enum Status { Active = "ACTIVE", Inactive = "INACTIVE" }  // string enum
const enum Flags { A = 1, B = 2 }              // const enum
```

### What needs to happen

Enums create:
1. A **type** (union of member literal types): `Direction` = `Direction.Up | Direction.Down | ...`
2. A **value** (object with member properties): `Direction.Up` = `0`

For v1, focus on numeric enums with auto-incrementing values.

### 2a. Add to `declared_type.rs`

In `resolve_declared_type`:

```rust
AstKind::TSEnumDeclaration(decl) => {
    self.get_type_of_enum_declaration(decl)
}
```

**Check the AstKind variant name**:
```bash
grep -n "Enum" crates/oxc_ast/src/generated/ast_kind.rs | head -10
```

New method:

```rust
fn get_type_of_enum_declaration(
    &mut self,
    decl: &oxc_ast::ast::TSEnumDeclaration<'_>,
) -> TypeId {
    // For v1: create a union of the member literal types.
    // Numeric enums: members are number literals (auto-incrementing from 0).
    // String enums: members are string literals.
    let mut member_types = Vec::new();
    let mut auto_value: f64 = 0.0;

    for member in &decl.members {
        let member_type = if let Some(init) = &member.initializer {
            // Has explicit initializer
            let init_type = self.get_type_of_expression(init);
            // Try to extract the numeric value for auto-increment tracking
            if let TypeData::Literal(LiteralType::Number(n)) =
                self.type_arena.get_data(init_type)
            {
                auto_value = *n + 1.0;
            }
            init_type
        } else {
            // Auto-incrementing numeric value
            let lit_type = self.type_arena.new_type(
                TypeFlags::NumberLiteral,
                ObjectFlags::None,
                TypeData::Literal(LiteralType::Number(auto_value)),
                None,
            );
            auto_value += 1.0;
            lit_type
        };
        member_types.push(member_type);
    }

    if member_types.is_empty() {
        return self.never_type;
    }

    self.get_or_create_union_type(member_types)
}
```

**Note**: This is a simplified model. Real tsc enum types are more complex (they have their own TypeFlags::Enum and EnumLiteral flags, and enum members are distinct types). But for conformance test impact, having enum declarations resolve to something other than `any` is the key win.

### 2b. Enum member access (depends on Track A)

Accessing `Direction.Up` requires `StaticMemberExpression` support (Track A). This track just handles the declaration. Once Track A implements property access, enum members could be accessed if the enum is modeled as an object with properties.

For better enum support, also add the enum as a value (object with member properties) in `resolve_symbol_type`:

```rust
AstKind::TSEnumDeclaration(decl) => {
    // Value side: an object with member properties
    let mut properties = IndexMap::new();
    let mut auto_value: f64 = 0.0;
    for member in &decl.members {
        if let Some(name) = member.id.static_name() {
            let member_type = if let Some(init) = &member.initializer {
                let t = self.get_type_of_expression(init);
                if let TypeData::Literal(LiteralType::Number(n)) =
                    self.type_arena.get_data(t)
                {
                    auto_value = *n + 1.0;
                }
                t
            } else {
                let t = self.type_arena.new_type(
                    TypeFlags::NumberLiteral,
                    ObjectFlags::None,
                    TypeData::Literal(LiteralType::Number(auto_value)),
                    None,
                );
                auto_value += 1.0;
                t
            };
            properties.insert(CompactStr::new(&name), member_type);
        }
    }
    self.type_arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Anonymous,
        TypeData::Object(ObjectType { target: None, properties }),
        None,
    )
}
```

**AST exploration**: Check enum member structure:
```bash
grep -n "TSEnumMember\|TSEnumDeclaration" crates/oxc_ast/src/generated/ast/ts.rs | head -20
```

Key things to verify:
- `member.id` — what type? Does it have `static_name()`? It's likely `TSEnumMemberName` which may differ from `PropertyKey`.
- `member.initializer` — `Option<Expression>`

### 2c. Add enum checking in `checker.rs`

Move `Statement::TSEnumDeclaration(_)` from catch-all:

```rust
Statement::TSEnumDeclaration(_) => {
    // Enum declarations are checked via their declared type resolution.
    // No body to check (unlike functions/classes).
    // TODO: check for duplicate member names, check initializer types
}
```

### Tests

```rust
#[test]
fn enum_numeric_basic() {
    with_checker!(
        "enum Direction { Up, Down, Left, Right }",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty());
        }
    );
}

#[test]
fn enum_as_type_annotation() {
    with_checker!(
        "enum Direction { Up, Down } let d: Direction = 0",
        |checker, program| {
            let diagnostics = checker.check_program(program);
            // Direction is union of 0 | 1, and 0 is assignable
            assert!(diagnostics.is_empty());
        }
    );
}
```

---

## Task 3: FunctionDeclaration types

### Current state

Function declarations ARE handled for **body checking** (`checker.rs:211-213`):
```rust
Statement::FunctionDeclaration(func) => {
    self.check_function_body(func);
}
```

But function declarations are NOT handled for **value-namespace type resolution**. When you write:
```typescript
function add(a: number, b: number): number { return a + b; }
let f = add;  // f should have type (a: number, b: number) => number
```

Currently `f` gets `any` because `resolve_symbol_type` doesn't handle `AstKind::Function`.

### What needs to happen

For v1, the simplest useful thing is to make function declarations resolve to `any` explicitly (already happens), but ideally we'd return a function type. However, function types require call signature infrastructure that doesn't exist yet.

**Pragmatic v1**: Add the arm but return `any`:

```rust
AstKind::Function(_) => {
    // TODO: create function type with call signature
    // Requires: parameter types, return type, type parameters
    self.any_type
}
```

**Stretch v1** (if time permits): Create an Object type that stores the return type somehow. This would at least let `let x: number = add()` work once CallExpression is implemented. But this is secondary to classes and enums.

### Check AstKind variant:
```bash
grep -n "Function\b" crates/oxc_ast/src/generated/ast_kind.rs | head -10
```

---

## Priority Order

1. **ClassDeclaration** — ~93 conformance tests, most impactful
2. **EnumDeclaration** — meaningful impact, enums are common in TS
3. **FunctionDeclaration value type** — low impact without call signature infrastructure

---

## Files Modified Summary

| File | Changes |
|------|---------|
| `crates/oxc_checker/src/declared_type.rs` | Add Class and Enum arms to `resolve_declared_type`, new methods |
| `crates/oxc_checker/src/expression_type.rs` | Add Class and Enum arms to `resolve_symbol_type` (value namespace) |
| `crates/oxc_checker/src/checker.rs` | Move ClassDeclaration/TSEnumDeclaration from catch-all, add `check_class_declaration` |
| `crates/oxc_checker/src/tests.rs` | Add tests (append at end) |

No new files needed. No changes to `type_data.rs`, `type_from_type_node.rs`, or `Cargo.toml`.

## Important: AST Exploration Before Coding

Before writing code, verify the AST shapes. The oxc AST is generated and the exact variant names/field names matter:

```bash
# Class structure
grep -n "pub struct Class\b" crates/oxc_ast/src/generated/ast/js.rs
grep -n "ClassElement\|PropertyDefinition\|MethodDefinition" crates/oxc_ast/src/generated/ast/js.rs | head -20

# Enum structure
grep -n "TSEnumDeclaration\|TSEnumMember\b" crates/oxc_ast/src/generated/ast/ts.rs | head -20

# AstKind variants
grep -n "Class\b\|TSEnum\|Function\b" crates/oxc_ast/src/generated/ast_kind.rs | head -20

# What PropertyDefinition fields are available
grep -A 20 "pub struct PropertyDefinition" crates/oxc_ast/src/generated/ast/js.rs | head -25
```

Read the actual struct definitions before implementing. The plan above uses best-guess field names based on standard oxc conventions, but verify them.
