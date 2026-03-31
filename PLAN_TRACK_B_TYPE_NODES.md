# Track B: Type Node Resolution (TSArrayType, TSFunctionType, TSTupleType, TSConstructorType)

## Parallel Work Notice

This track runs IN PARALLEL with:
- **Track A** (expression types: StaticMemberExpression, CallExpression, ArrayExpression)
- **Track C** (declaration types: ClassDeclaration, EnumDeclaration, FunctionDeclaration)

All three tracks modify different files/methods. The ONLY shared surface is:
- `type_from_type_node.rs` — **this track owns it**. Tracks A and C should NOT modify this file.
- `type_display.rs` — all tracks may need to add display arms. Coordinate by adding new `TypeData::*` match arms only (don't touch existing arms). Each track adds its own arms.
- `type_data.rs` — already has all needed struct definitions. No modifications needed.
- `tests.rs` — each track appends tests at the end of the file. No conflicts if appending only.

## Codebase Overview

### Repository: `/Users/leoshatrushin/dev/oxc` (branch: `oxc-typecheck`)
### Crate: `crates/oxc_checker`

### Architecture
- **TypeArena** (`crates/oxc_types/src/type_arena.rs`): SoA storage with `IndexVec<TypeId, T>` columns for flags, object_flags, data, symbols.
- **TypeId**: u32 newtype index into the arena.
- **TypeData** (`crates/oxc_types/src/type_data.rs`): Enum with 19 variants. Each type in the arena has flags + object_flags + data + optional symbol.
- **TypeFlags** (`crates/oxc_types/src/type_flags.rs`): u32 bitflags classifying type kinds.
- **ObjectFlags** (`crates/oxc_types/src/object_flags.rs`): u32 bitflags for Object/Union/Intersection subtypes.

### Key method: `type_arena.new_type(flags, object_flags, data, symbol) -> TypeId`

### How types are created from type annotations
All type annotation resolution goes through `get_type_from_type_node(&mut self, ts_type: &TSType) -> TypeId` in `type_from_type_node.rs`. It's a big match on `TSType` variants. Currently implemented:
- All keyword types (string, number, boolean, any, etc.)
- TSParenthesizedType (unwraps)
- TSUnionType
- TSIntersectionType
- TSLiteralType (string/number/bigint/boolean literals)
- TSTypeReference (with generic instantiation)
- TSTypeLiteral (`{ x: number; y: string }`)

Currently returning `self.any_type` (catch-all):
- **TSArrayType** ← implement this
- **TSFunctionType** ← implement this
- **TSTupleType** ← implement this
- **TSConstructorType** ← implement this
- TSConditionalType, TSImportType, TSIndexedAccessType, TSInferType, TSMappedType, TSNamedTupleMember, TSTemplateLiteralType, TSThisType, TSTypeOperatorType, TSTypePredicate, TSTypeQuery, JSDoc types

### How to test
```bash
cd /Users/leoshatrushin/dev/oxc && cargo test -p oxc_checker
```

### Test pattern (from `tests.rs`)
```rust
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

// Helper to get type annotation from first var decl:
fn first_var_type_annotation<'a>(program: &'a Program<'a>) -> Option<&'a TSType<'a>> { ... }
```

### type_display.rs pattern
Currently has match arms for: Intrinsic, Literal, Union, Intersection, Object, Interface, catch-all `_ => "{...}"`. Add new arms BEFORE the `_ =>` catch-all.

---

## Task 1: TSArrayType (`number[]`, `string[]`)

### What it is
`number[]` is syntactic sugar for `Array<number>`. In the AST it's `TSArrayType { element_type: TSNumberKeyword }`.

### Implementation in `type_from_type_node.rs`

Move `TSType::TSArrayType(_)` out of the catch-all. Add:

```rust
TSType::TSArrayType(arr) => {
    let element_type = self.get_type_from_type_node(&arr.element_type);
    // Look up the global Array interface and create a TypeReference
    let array_type = self.get_global_type("Array");
    if array_type == self.any_type {
        // No Array type available — fallback to any
        return self.any_type;
    }
    self.type_arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Reference,
        TypeData::TypeReference(TypeReferenceType {
            target: Some(array_type),
            resolved_type_arguments: SmallVec::from_buf([element_type]),
        }),
        None,
    )
},
```

Add to imports at top of file: make sure `TypeReferenceType` is imported from `oxc_types`. Also `SmallVec` from `smallvec`.

Check existing imports — file already imports `TypeReferenceType` and `SmallVec`.

### Display in `type_display.rs`

TypeReference display needs to be added. When the target is the global `Array` type, display as `T[]` format. Otherwise display as `TypeName<Args>`.

For now, add a `TypeData::TypeReference` arm:
```rust
TypeData::TypeReference(tr) => {
    // For now, display as the resolved properties (structural)
    // TODO: named display like Array<string> or string[]
    if let Some(target) = tr.target {
        let target_str = self.type_to_string(target);
        if tr.resolved_type_arguments.is_empty() {
            target_str
        } else {
            let args = tr.resolved_type_arguments
                .iter()
                .map(|&t| self.type_to_string(t))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", target_str, args)
        }
    } else {
        "{...}".to_string()
    }
}
```

**Important**: tsc displays `Array<string>` as `string[]` in `.types` baselines. Check what format the conformance tests expect. If they expect `string[]`, the display logic should detect when target is the Array interface and use the shorthand. The target's name can be checked by looking at the InterfaceType/Intrinsic name, or by comparing against `self.get_global_type("Array")`. Since `type_to_string` takes `&self` (not `&mut self`), you can't call `get_global_type` there. Instead, check the target's data — if it's an `InterfaceType` or `Intrinsic` with name matching "Array", use shorthand.

Actually, looking more carefully: `type_to_string` takes `&self`, and `get_global_type` also takes `&self`. So you CAN call it. Use:
```rust
// In TypeReference display, after getting target:
// Check if this is Array<T> — display as T[]
// (get_global_type is &self, type_to_string is &self, so this works)
```

Wait — `get_global_type` is on `Checker`, not on the display impl. The display IS on `Checker`. So yes, `self.get_global_type("Array")` works from `type_to_string`.

### Tests
```rust
#[test]
fn array_type_annotation() {
    with_checker!("let x: number[]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        let flags = checker.type_arena().get_flags(type_id);
        assert!(flags.intersects(TypeFlags::Object));
        // Display format depends on tsc convention
    });
}

#[test]
fn array_type_assignability() {
    with_checker!(
        "let x: number[] = [1, 2, 3]",
        |checker, program| {
            // ArrayExpression is Track A's job, so this will still be `any` for now
            // But the type annotation itself should resolve
            let diagnostics = checker.check_program(program);
            assert!(diagnostics.is_empty()); // any is assignable to anything
        }
    );
}
```

---

## Task 2: TSFunctionType (`(x: number) => string`)

### What it is
Function types in type position: `(a: number, b: string) => boolean`. In the AST: `TSFunctionType { params, return_type, type_parameters }`.

### TypeData consideration
There's no `FunctionType` variant in TypeData currently. Function types in tsc are represented as Object types with call signatures. For a simple v1:

**Option A (recommended)**: Create the function type as an Object type with a special marker. Since we don't have call signature infrastructure yet, store it as an Object type and return `any_type` for the overall type but resolve the return type. This is the pragmatic approach.

**Option B (proper)**: Add call signature support to ObjectType. This is more work but more correct.

**Recommended approach for v1**: Return a simple representation. The most useful thing is that `() => number` should be assignable to `() => number`. For conformance tests, what matters is that the type _displays_ correctly.

Looking at tsc's `.types` output, function types display as `(x: number) => string`.

For v1, create a new TypeData variant or use Object with metadata. Actually, let's look at what tsc does: function types are `ObjectType` with `callSignatures`. We don't have that infrastructure.

**Simplest useful approach**:
1. Resolve the return type (useful for checking)
2. Store as a new simple struct or just use `any_type` with a TODO
3. At minimum, handle the display

Actually, for maximum conformance test wins with minimal effort:
- The type should display correctly in `type_to_string`
- It needs to be distinguishable from other types

**Pragmatic v1**: Create a minimal `FunctionType` or use the existing Object infrastructure. Since many conformance tests just check display output, getting the display right is the priority.

Let me suggest a clean approach:

Add a simple wrapper. In `type_from_type_node.rs`:

```rust
TSType::TSFunctionType(func) => {
    // For now, resolve to `any` — function type checking requires
    // call signature infrastructure
    // TODO: implement proper function types with call signatures
    self.any_type
},
```

This is the honest v1. Function types need call signatures to be useful, and that's a bigger piece of infrastructure. Just moving it out of the catch-all with a dedicated arm (even if it returns `any`) is good for code organization.

If you want more impact: parse the return type at least, so `let f: () => number; let x: number = f()` could work once CallExpression is implemented (Track A). But that requires storing the return type somewhere.

**Stretch goal**: Add a `FunctionTypeData` struct to `TypeData` if you want to store params + return type. But this is secondary to TSArrayType and TSTupleType which have more conformance impact.

---

## Task 3: TSTupleType (`[string, number]`)

### What it is
Tuple types: `[string, number, boolean]`. In the AST: `TSTupleType { element_types: Vec<TSTupleElement> }`.

Each element can be:
- A plain type: `string`
- A named element: `name: string` (TSNamedTupleMember)
- Optional: `string?`
- Rest: `...string[]`

### TypeData
`TupleType` already exists in `type_data.rs`:
```rust
pub struct TupleType {
    pub target: Option<TypeId>,
    pub resolved_type_arguments: SmallVec<[TypeId; 4]>,
    pub element_infos: Vec<TupleElementInfo>,
    pub min_length: u32,
    pub fixed_length: u32,
    pub combined_flags: ElementFlags,
    pub readonly: bool,
}

pub struct TupleElementInfo {
    pub element_type: TypeId,
    pub flags: ElementFlags,
    pub label_name: Option<CompactStr>,
}
```

### Implementation in `type_from_type_node.rs`

```rust
TSType::TSTupleType(tuple) => {
    self.get_type_from_tuple_type_node(tuple)
},
```

New method:
```rust
fn get_type_from_tuple_type_node(
    &mut self,
    tuple: &oxc_ast::ast::TSTupleType<'_>,
) -> TypeId {
    let mut element_infos = Vec::new();
    let mut type_arguments = SmallVec::new();
    let mut min_length: u32 = 0;
    let mut has_rest = false;

    for element in &tuple.element_types {
        let (elem_type, flags, label) = self.resolve_tuple_element(element);
        type_arguments.push(elem_type);
        if flags.contains(ElementFlags::Required) {
            min_length += 1;
        }
        if flags.contains(ElementFlags::Rest) {
            has_rest = true;
        }
        element_infos.push(TupleElementInfo {
            element_type: elem_type,
            flags,
            label_name: label,
        });
    }

    let fixed_length = if has_rest {
        type_arguments.len() as u32 - 1
    } else {
        type_arguments.len() as u32
    };

    let combined_flags = element_infos
        .iter()
        .fold(ElementFlags::empty(), |acc, info| acc | info.flags);

    self.type_arena.new_type(
        TypeFlags::Object,
        ObjectFlags::Tuple,
        TypeData::Tuple(TupleType {
            target: None,
            resolved_type_arguments: type_arguments,
            element_infos,
            min_length,
            fixed_length,
            combined_flags,
            readonly: false, // TODO: handle readonly tuples
        }),
        None,
    )
}

fn resolve_tuple_element(
    &mut self,
    element: &oxc_ast::ast::TSTupleElement<'_>,
) -> (TypeId, ElementFlags, Option<CompactStr>) {
    use oxc_ast::ast::TSTupleElement;
    match element {
        TSTupleElement::TSNamedTupleMember(named) => {
            let elem_type = self.get_type_from_type_node(&named.element_type);
            let flags = if named.optional {
                ElementFlags::Optional
            } else {
                ElementFlags::Required
            };
            let label = Some(CompactStr::new(named.label.name.as_str()));
            (elem_type, flags, label)
        }
        TSTupleElement::TSOptionalType(opt) => {
            let elem_type = self.get_type_from_type_node(&opt.type_annotation);
            (elem_type, ElementFlags::Optional, None)
        }
        TSTupleElement::TSRestType(rest) => {
            let elem_type = self.get_type_from_type_node(&rest.type_annotation);
            (elem_type, ElementFlags::Rest, None)
        }
        // All other cases are plain type nodes
        _ => {
            // TSTupleElement variants that are just TSType wrappers
            let ts_type: &TSType = element.to_ts_type();
            let elem_type = self.get_type_from_type_node(ts_type);
            (elem_type, ElementFlags::Required, None)
        }
    }
}
```

**IMPORTANT**: Check the actual oxc AST for `TSTupleElement`. It may be a different enum shape. Read the AST definitions:
- `crates/oxc_ast/src/generated/ast/ts.rs` — search for `TSTupleElement` and `TSTupleType`
- The element variants may need different handling

Run this to find the definition:
```bash
grep -n "TSTupleElement\|enum TSTupleElement" crates/oxc_ast/src/generated/ast/ts.rs | head -20
```

Also check if `TSTupleElement` has a `to_ts_type()` method or if you need to match each variant individually.

### Imports needed
```rust
use oxc_types::{TupleType, ElementFlags, TupleElementInfo};
```

### Display in `type_display.rs`

Add before the catch-all:
```rust
TypeData::Tuple(tuple) => {
    let elements = tuple.element_infos
        .iter()
        .map(|info| {
            let type_str = self.type_to_string(info.element_type);
            if let Some(label) = &info.label_name {
                if info.flags.contains(oxc_types::ElementFlags::Optional) {
                    format!("{}?: {}", label, type_str)
                } else if info.flags.contains(oxc_types::ElementFlags::Rest) {
                    format!("...{}: {}", label, type_str)
                } else {
                    format!("{}: {}", label, type_str)
                }
            } else if info.flags.contains(oxc_types::ElementFlags::Optional) {
                format!("{}?", type_str)
            } else if info.flags.contains(oxc_types::ElementFlags::Rest) {
                format!("...{}", type_str)
            } else {
                type_str
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", elements)
}
```

### Tests
```rust
#[test]
fn tuple_type_basic() {
    with_checker!("let x: [string, number]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "[string, number]");
    });
}

#[test]
fn tuple_type_named() {
    with_checker!("let x: [name: string, age: number]", |checker, program| {
        let ts_type = first_var_type_annotation(program).unwrap();
        let type_id = checker.get_type_from_type_node(ts_type);
        assert_eq!(checker.type_to_string(type_id), "[name: string, age: number]");
    });
}
```

---

## Task 4: TSConstructorType (`new (x: number) => Foo`)

Low priority — same infrastructure gap as TSFunctionType (needs call/construct signatures). For v1, just move it out of the catch-all:

```rust
TSType::TSConstructorType(_) => {
    // TODO: implement constructor types (requires construct signatures)
    self.any_type
},
```

---

## Priority Order

1. **TSArrayType** — highest conformance impact, straightforward
2. **TSTupleType** — good conformance impact, TypeData already exists
3. **TSFunctionType** — move out of catch-all, return `any` for now
4. **TSConstructorType** — move out of catch-all, return `any` for now

---

## Also handle: TSNamedTupleMember

Currently in the catch-all. Move it out and handle it within the tuple resolution (it's a TSTupleElement, not a standalone TSType in practice, but the AST may have it as both):

```rust
TSType::TSNamedTupleMember(_) => {
    // Handled as part of TSTupleType resolution
    self.any_type
},
```

---

## Files Modified Summary

| File | Changes |
|------|---------|
| `crates/oxc_checker/src/type_from_type_node.rs` | Add TSArrayType, TSTupleType handlers; move TSFunctionType/TSConstructorType out of catch-all |
| `crates/oxc_checker/src/type_display.rs` | Add TypeReference and Tuple display arms |
| `crates/oxc_checker/src/tests.rs` | Add tests (append at end) |

No new files. No changes to `type_data.rs`, `Cargo.toml`, or any other files.

## Verification
```bash
cd /Users/leoshatrushin/dev/oxc && cargo test -p oxc_checker
cd /Users/leoshatrushin/dev/oxc && cargo run -p oxc_coverage -- checker 2>&1 | tail -10
```
