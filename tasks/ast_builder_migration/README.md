# `AstBuilder` migration codemod

[ast-grep](https://ast-grep.github.io) rules that do two things:

1. **Migrate** code from the old `AstBuilder` (methods on the `AstBuilder` struct,
   e.g. `self.ast.null_literal(span)`) to the new builder methods defined directly on AST types
   (e.g. `NullLiteral::new(span, self)`).

2. **Shorten** the result, collapsing verbose new-builder call shapes into their shorthand
   equivalents (e.g. `Statement::VariableDeclaration(VariableDeclaration::boxed(..))` ->
   `Statement::new_variable_declaration(..)`).

See <https://github.com/oxc-project/oxc/issues/23043> for background.

**Oxc's own crates are already migrated.** This codemod is kept for **downstream consumers** of
Oxc crates who built on the old `AstBuilder` and now need to move to the new builder methods.
The defaults in [`generate_rules.mts`](generate_rules.mts) reproduce the rules Oxc used.
Before running it against your own repo, adjust the config consts to match your conventions -
see [Configuring for your repo](#configuring-for-your-repo).

## How it works

1. `AstBuilderGenerator` (in `tasks/ast_tools`) emits two data files, regenerated with `just ast`:
   - [`generated/mappings.json`](generated/mappings.json) maps each old `AstBuilder` method name to
     the equivalent new method on the AST type, e.g. `null_literal` -> `NullLiteral::new`,
     `alloc_null_literal` -> `NullLiteral::boxed`, `statement_expression` ->
     `Statement::new_expression_statement`.
   - [`generated/shorten_mappings.json`](generated/shorten_mappings.json) holds the structural data
     the shortening rules need: which structs are boxable, each enum variant's inner builder, and the
     enum inheritance graph.

   Generating both from the same code that builds the method names means they capture the name
   de-duplication, reserved-word, and `_with_*` default-field quirks exactly.

2. [`custom_mappings.json`](custom_mappings.json) adds, by hand, the methods that are written by hand
   rather than emitted by codegen (e.g. `void_0` -> `Expression::new_void_0`).

3. [`generate_rules.mts`](generate_rules.mts) combines these and writes
   [`generated/rules.yml`](generated/rules.yml) - the migration rules (one per builder method) plus
   the shortening rules.

4. Apply the rules to a crate (or any path) with [`run.sh`](run.sh):

   ```sh
   tasks/ast_builder_migration/run.sh <path>
   ```

   All arguments are forwarded to `ast-grep scan`.
   `ast-grep` applies a single pass per invocation and does not re-scan its own output, but the rules
   cascade (one rule's output is another's input, and nested calls collapse one level per pass),
   so `run.sh` re-runs `ast-grep scan --update-all` until a pass makes no changes.

   Exclude any generated or vendored code that defines/uses the builder methods themselves (the
   shortening rules are derived from those shapes and would rewrite them), e.g.
   `run.sh src --globs '!**/generated/**'`. Run your formatter afterwards to tidy the output.

## Configuring for your repo

The defaults reproduce Oxc's own rules. If your codebase uses different conventions, edit the
config consts at the top of [`generate_rules.mts`](generate_rules.mts), then regenerate and apply:

```sh
node tasks/ast_builder_migration/generate_rules.mts   # rewrites generated/rules.yml
tasks/ast_builder_migration/run.sh <path>
```

The consts:

1. **`PRIMITIVE_TYPES`**: Names of the arena primitive types. Oxc uses `ArenaBox`, `ArenaVec`,
   `Ident`, and `Str` (these appear in the custom mappings and the box-collapse rule). If your repo
   imports them under other names, map Oxc's name -> yours, e.g.
   `{ ArenaVec: "Vec" }`, `{ ArenaVec: "OxcVec" }`, or `{ ArenaVec: "oxc_allocator::Vec" }`.
   Only these exact names are renamed; AST type names (`Expression`, `NullLiteral`, ...) are untouched.

2. **`AST_BUILDER_FIELD`**: The field the old `AstBuilder` is reached through. Oxc writes
   `self.ast.null_literal(span)`, so it is `"ast"` and the rules match `$ACCESSOR.ast.<method>(...)`.
   - For `ctx.builder.null_literal(span)`, set it to `"builder"`.
   - If you hold the builder/allocator directly in a variable (`ast.null_literal(span)`, no field access),
     set it to `""`. The rules then match `$ACCESSOR.<method>(...)`.

3. **`ACCESSOR_BY_REF`**: Whether the appended accessor argument is passed by reference. Oxc appends
   it bare (`NullLiteral::new(span, self)`) because `self` implements `GetAstBuilder`/`GetAllocator`,
   so it is `false`. If the accessor is a variable the new methods take by reference, set it to
   `true` to get `NullLiteral::new(span, &ast)`.

4. **`AST_TYPE_PREFIX`**: The module path AST types are referred to through. All rules use bare type
   names (`Expression::new_*`, `NullLiteral::new`, ...), matching Oxc's `use oxc_ast::ast::*;`. If
   your code reaches the types through a module alias (as `oxc_react_compiler` does, via
   `use oxc_ast::ast as oxc;` -> `oxc::Expression`), set this to that prefix *including the trailing
   `::`*, e.g. `"oxc::"`. It is prepended to every AST type name, but not to the arena primitives
   (those carry their own paths via `PRIMITIVE_TYPES`).

The field/by-ref consts (2, 3) only affect the **migration** rules. The type-naming consts (1, 4)
affect both: the migration fixes emit the renamed/prefixed types, and the **shortening** rules match
and rewrite those same shapes.

## Migration rules

Each migration rule rewrites a call on the old builder to a call to the new method, appending
the accessor (the base of the `<accessor>.ast` receiver) as the final argument:

```rs
self.ast.null_literal(span)             // -> NullLiteral::new(span, self)
p.ast.alloc_object_expression(span, x)  // -> ObjectExpression::boxed(span, x, p)
self.ast.number_0()                     // -> Expression::new_number_0(self)
```

The accessor (`self` / `p`) must implement `GetAstBuilder` and `GetAllocator`.

## Shortening rules

These run on already-migrated code, collapsing verbose shapes into the shorthand constructors.
They are purely structural - they never mention `<accessor>.ast` - and come in three kinds:

```rs
// box-collapse: ArenaBox::new_in(T::new(..), x)  ->  T::boxed(..)
ArenaBox::new_in(VariableDeclaration::new(span, kind, decls, declare, b), x)
// -> VariableDeclaration::boxed(span, kind, decls, declare, b)

// variant-wrap: E::Variant(Inner::boxed(..))  ->  E::new_variant(..)
Statement::VariableDeclaration(VariableDeclaration::boxed(span, kind, decls, declare, b))
// -> Statement::new_variable_declaration(span, kind, decls, declare, b)

// inherited-from: Outer::from(Inner::new_x(..))  ->  Outer::new_x(..)
Statement::from(Declaration::new_variable_declaration(span, kind, decls, declare, b))
// -> Statement::new_variable_declaration(span, kind, decls, declare, b)
```

`variant-wrap` also covers inherited variants (e.g. `Statement::VariableDeclaration`, inherited from
`Declaration`). `box-collapse` discards the outer allocator argument (`x` above); in real code it is
always the same value as the inner builder argument, so the rewrite is behaviour-preserving.

## Reuse by downstream consumers

Only the generated `rules.yml` is Oxc-specific; the generator and script are reusable. Point them at
your repo by editing the config consts (see [Configuring for your repo](#configuring-for-your-repo))
and regenerating - no need to hand-edit the rule templates.
