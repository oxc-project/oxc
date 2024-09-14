# Oxc transformer

We mostly model Oxc's transforms on [Babel](https://babeljs.io/)'s implementations.

First iteration of a transform will usually be as straight a port from Babel as possible. We may then
iterate from there to gain better performance.

We import Babel's transformer tests, and aim to pass them all.

All transforms are implementations of the `Traverse` trait.

## Composing transforms

We aim to run all transforms together **in a single AST visitation pass**.

This will be the most performant method, though it causes some complexity, especially due to interactions
between different transforms acting on the same code. It is unclear at present if this methodology will
be viable, but it is our initial aim, and we will only fall back to multiple passes if single-pass proves
unworkable.

## Style guide for implementing transforms

Transforms are complex. Please try to make the code as clear and easy to follow as possible.

NB: Not all the "rules" in this style guide are currently followed in transforms we've written so far.
We will update those transforms to follow this guide when we have time. But all new transforms should
follow this style guide closely.

### Structure

Each transform should be in its own file.

Some transforms just delegate work to sub-transforms. e.g. `React` transform delegates to the `ReactJsx`
and `ReactDisplayName` transforms.

### Comments

For a maintainable and understandable codebase, please go big on code comments. The more, the merrier!

#### Top of file

Each transform should include a comment at top of file including:

- High level explanation of what transform does.
- One "before / after" example.
- Link to Babel plugin.
- Note of any ways in which our implementation diverges from Babel's, and why.

#### Methods

If it's a complicated transform with multiple visitors which interact with each other, add comments
explaining how the pieces fit together.

#### Code snippets

`AstBuilder` calls are often very verbose. Preface each chunk of `AstBuilder` calls with a short comment
showing what this code produces. e.g.:

```rs
// `let Foo;`
let declarations = {
    let ident = BindingIdentifier::new(SPAN, "Foo");
    let pattern_kind = self.ast.binding_pattern_identifier(ident);
    let binding = self.ast.binding_pattern(pattern_kind, None, false);
    let decl = self.ast.variable_declarator(SPAN, VariableDeclarationKind::Let, binding, None, false);
    self.ast.new_vec_single(decl)
};
let var_decl = Declaration::VariableDeclaration(self.ast.variable_declaration(
    SPAN,
    kind,
    declarations,
    Modifiers::empty(),
));
```

#### Where we can improve on Babel

Babel has less of an emphasis on performance than Oxc has. For this reason Babel's implementations are
often not as efficient as they could be.

In some cases, we could do better, but we are unable to at present because a more efficient
implementation would result in cosmetic differences between Oxc's output and Babel's (e.g. different
variable names) which causes Babel's tests to fail when run on Oxc's output.

In future we may find a way to work around this problem.

So where we feel Babel's implementation is inefficient, but we have to follow it at present to pass their
tests, make a `// TODO(improve-on-babel): Babel's impl is inefficient because X, we could do better by Y`
comment, so we can return to it later.

### Clear "entry points"

"Entry points" are where the visitor calls into the transform.

- Entry points of transform should be implemented as `impl Traverse for MyTransform`.
- Those methods have to be called `enter_*` and `exit_*`.
- Parent transform will only interface with child transform via these entry points.
- Only other method exposed externally should be `new`. That should be at top of the file.
- Entry points go directly below `new` method definition.
- Internal methods implemented lower down in an `impl MyTransform` block.
- Internal methods named descriptively - `add_id_to_function` not `transform_function`.

i.e. File is laid out so logic flows from top of file to bottom.

e.g.:

```rs
struct FunctionRenamer {
    prefix: String,
}

// Initialization
impl FunctionRenamer {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

// Entry points
impl<'a> Traverse<'a> for FunctionRenamer {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                self.rename_function(func, ctx);
            }
            Statement::ExportDefaultDeclaration(decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &mut decl.declaration {
                    self.rename_function(func, ctx);
                }
            }
        }
    }
}

// Internal methods
impl FunctionRenamer {
    /// Rename the function
    // This function's name describes what it does, not just `transform_function`
    fn rename_function(&mut self, func: &mut Function<'a>, ctx: &mut TraverseCtx<'a>) {
        // Do stuff
    }
}
```

### Encapsulate logic

All logic for each transform should live in that specific file, with no "leaking" into the parent
transform. Each transform is only called into via the standard `enter_*`/`exit_*` entry points.

Only exception is that parent can check if child transform is enabled or not.

#### Bad! Don't do this.

Here some of logic from child transform is "leaked" into the parent:

```rs
// src/do_stuff/mod.rs
impl<'a> Traverse<'a> for ParentTransform {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.child_is_enabled {
            match expr {
                Expression::JSXElement(e) => {
                    self.child.enter_jsx_element(e, ctx);
                }
                Expression::JSXFragment(e) => {
                    self.child.enter_jsx_fragment(e, ctx);
                }
                _ => {}
            }
        }
    }
}

// src/do_stuff/child.rs
impl<'a> Traverse<'a> for ChildTransform {
    fn enter_jsx_element(&mut self, elem: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Do stuff
    }
    fn enter_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a>) {
        // Do stuff
    }
}
```

#### Good!

All the child transform's logic is encapsulated in `ChildTransform`:

```rs
// src/do_stuff/mod.rs
impl<'a> Traverse<'a> for ParentTransform {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.child_is_enabled {
            self.child.enter_expression(expr, ctx);
        }
    }
}

// src/do_stuff/child.rs
impl<'a> Traverse<'a> for ChildTransform {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::JSXElement(e) => {
                self.do_stuff_to_jsx_element(e, ctx);
            }
            Expression::JSXFragment(e) => {
                self.do_stuff_to_jsx_fragment(e, ctx);
            }
            _ => {}
        }
    }
}

impl ChildTransform {
    fn do_stuff_to_jsx_element(&mut self, elem: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Do stuff
    }
    fn do_stuff_to_jsx_fragment(&mut self, elem: &mut JSXFragment<'a>, ctx: &mut TraverseCtx<'a>) {
        // Do stuff
    }
}
```
