# ember-eslint-parser AST Analysis

## Key Findings

### 1. Parser Output Structure

`ember-eslint-parser.parseForESLint()` returns:

- **`ast`**: Standard ESTree AST with Glimmer extensions
- **`scopeManager`**: Scope analysis (13-17 scopes for our samples)
- **`services`**: TypeScript services (program, node maps)
- **`visitorKeys`**: Custom visitor keys (190 keys)
- **`isTypescript`**: Boolean flag for TS detection

### 2. Template Representation

The `<template>` blocks are represented as **`GlimmerTemplate`** nodes embedded in the class body:

```javascript
ClassDeclaration {
  body: {
    body: [
      PropertyDefinition,  // @tracked count = 0
      MethodDefinition,    // @action increment()
      MethodDefinition,    // @action decrement()
      GlimmerTemplate {    // <template>...</template>
        type: "GlimmerTemplate",
        body: [ /* Glimmer AST nodes */ ],
        loc: { ... },
        range: [ ... ]
      }
    ]
  }
}
```

### 3. Glimmer-Specific Node Types

The parser introduces **custom node types** not in standard ESTree:

#### Template Structure Nodes:

- **`GlimmerTemplate`** - The root `<template>` block
- **`GlimmerElementNode`** - HTML elements (`<div>`, `<button>`, etc.)
- **`GlimmerTextNode`** - Text content
- **`GlimmerAttrNode`** - HTML attributes
- **`GlimmerElementNodePart`** - Element parts

#### Template Expression Nodes:

- **`GlimmerMustacheStatement`** - `{{expression}}`
- **`GlimmerBlockStatement`** - `{{#if}}...{{/if}}`
- **`GlimmerPathExpression`** - Property paths (`this.count`)
- **`GlimmerStringLiteral`** - String literals in templates
- **`GlimmerHash`** - Named parameters
- **`GlimmerBlock`** - Block content

#### Template Modifier Nodes:

- **`GlimmerElementModifierStatement`** - `{{on "click" handler}}`

### 4. JavaScript/TypeScript Parts

The **non-template code** is standard ESTree/TS-ESTree:

- `ImportDeclaration` (standard)
- `ClassDeclaration` (standard)
- `MethodDefinition` (standard)
- `TSInterfaceDeclaration` (TS-ESTree)
- Decorators (`@tracked`, `@action`) - standard

### 5. Token Handling

The AST includes:

- **133 tokens** (GJS file)
- **217 tokens** (GTS file)
- **Glimmer tokens** mixed with JS tokens in the token stream

## Critical Challenges for oxc Integration

### Challenge 1: Custom Node Types

**Problem**: Our ESTree converter in `crates/oxc_linter/src/estree_converter.rs` doesn't know about Glimmer nodes.

**Options**:

1. **Strip Glimmer nodes** - Remove them before conversion
2. **Convert to comments** - Preserve location but make inert
3. **Create oxc equivalents** - Extend oxc AST (complex)
4. **Pass through as-is** - Keep for JS plugin rules

**Recommendation**: **Option 2 for Rust rules** (strip/comment out), **Option 4 for JS plugin rules** (pass original AST)

### Challenge 2: Visitor Keys

The parser provides **190 custom visitor keys** for traversing Glimmer nodes.

**Problem**: oxc's semantic analyzer won't know how to traverse Glimmer nodes.

**Solution**: Strip Glimmer nodes before semantic analysis, only analyze JS/TS parts.

### Challenge 3: Scope Manager

The parser provides a custom scope manager that understands template scoping.

**Problem**: Should we use it or rebuild with oxc's semantic analyzer?

**Recommendation**: **Use parser's scope manager for JS plugins**, **rebuild for Rust rules** (semantic analysis).

### Challenge 4: Two Rule Execution Paths

We need **TWO separate execution paths**:

#### Path A: Rust Built-in Rules

1. Parse with `ember-eslint-parser`
2. **Strip Glimmer nodes** from AST
3. Convert cleaned ESTree → oxc AST
4. Run oxc semantic analysis
5. Execute Rust rules (they only see JS/TS)

#### Path B: JavaScript Plugin Rules (eslint-plugin-ember)

1. Parse with `ember-eslint-parser`
2. **Keep full AST** with Glimmer nodes
3. Pass to JS plugin environment
4. Execute JS rules (they see templates)

## Detailed AST Structure Example

### GJS File Structure (sample.gjs)

```
Program
├── ImportDeclaration (from '@glimmer/component')
├── ImportDeclaration (from '@glimmer/tracking')
├── ImportDeclaration (from '@ember/object')
└── ExportDefaultDeclaration
    └── ClassDeclaration (CounterComponent)
        └── ClassBody
            ├── PropertyDefinition (@tracked count = 0)
            ├── MethodDefinition (increment)
            ├── MethodDefinition (decrement)
            └── GlimmerTemplate ← CUSTOM NODE TYPE!
                └── GlimmerElementNode (<div>)
                    └── GlimmerElementNode (<div class="counter">)
                        ├── GlimmerElementNode (<h2>)
                        │   ├── GlimmerTextNode ("Counter: ")
                        │   └── GlimmerMustacheStatement ({{this.count}})
                        ├── GlimmerElementNode (<button>)
                        │   ├── GlimmerAttrNode (type="button")
                        │   ├── GlimmerElementModifierStatement ({{on "click" ...}})
                        │   └── GlimmerTextNode ("Increment")
                        └── GlimmerElementNode (<button>)
                            └── ... similar structure
```

### GTS File Structure (sample.gts)

Similar to GJS but adds:

- `TSInterfaceDeclaration` (Signature interface)
- Type annotations on properties/methods
- `GlimmerBlockStatement` for `{{#if}}` blocks

## Implementation Strategy

### Phase 1: Basic Support (Strip Templates)

**Goal**: Get Rust rules working by stripping templates

```javascript
// In JS-side preprocessor (before passing to Rust)
function stripGlimmerNodes(ast) {
  // Replace GlimmerTemplate nodes with empty blocks or comments
  traverse(ast, {
    GlimmerTemplate(node) {
      // Replace with comment preserving location
      return {
        type: 'ExpressionStatement',
        expression: {
          type: 'Literal',
          value: '<template removed for Rust analysis>',
          range: node.range,
          loc: node.loc
        }
      };
    }
  });
  return ast;
}
```

**Benefits**:

- Rust rules work immediately
- Standard JS/TS code is linted correctly
- No oxc AST changes needed

**Limitations**:

- Rust rules can't analyze templates
- Template-aware rules must be in JS

### Phase 2: JS Plugin Pass-Through

**Goal**: Pass full AST (with Glimmer nodes) to JS plugins

```rust
// In oxlint Rust code
if has_js_plugins {
    // Pass original AST with Glimmer nodes
    let original_ast_json = parse_result.ast_json;
    execute_js_plugins(original_ast_json, /* ... */);
}
```

**Benefits**:

- eslint-plugin-ember rules work fully
- Template-aware rules function correctly
- Leverage existing ESLint ecosystem

### Phase 3: Hybrid Execution (Recommended)

**Implement both paths simultaneously:**

```rust
// Parse with custom parser
let parse_result = call_ember_eslint_parser(source_code);

// Path A: Rust rules (stripped AST)
let cleaned_ast = strip_glimmer_nodes(parse_result.ast);
let oxc_ast = estree_to_oxc(cleaned_ast);
let semantic = build_semantic(oxc_ast);
run_rust_rules(semantic);

// Path B: JS rules (full AST)
if has_js_plugins {
    execute_js_plugins(
        parse_result.ast,        // Full AST with Glimmer nodes
        parse_result.scopeManager,
        parse_result.services,
        parse_result.visitorKeys
    );
}
```

## Updated visitorKeys Handling

The parser provides custom visitor keys that tell ESLint how to traverse Glimmer nodes:

```json
{
  "GlimmerTemplate": ["body"],
  "GlimmerElementNode": ["path", "attributes", "modifiers", "children"],
  "GlimmerMustacheStatement": ["path", "params", "hash"]
  // ... 187 more entries
}
```

**For JS plugins**: Pass these visitor keys so ESLint traversal works correctly.
**For Rust rules**: Ignore them (we've stripped Glimmer nodes).

## Performance Considerations

### Token Count Impact

- GJS: 133 tokens
- GTS: 217 tokens
- Many are Glimmer-specific

**Impact**: Stripping Glimmer nodes will reduce token count by ~30-50%.

### Scope Manager

- GJS: 13 scopes
- GTS: 17 scopes

**Question**: Should we rebuild scopes with oxc or reuse parser's scope manager?

**Recommendation**:

- **Rebuild for Rust rules** (oxc semantic analysis)
- **Reuse for JS rules** (pass parser's scope manager)

## Next Steps

1. ✅ **Understand AST structure** (DONE)
2. ⏳ **Implement Glimmer node stripper** (JS-side)
3. ⏳ **Test stripped AST conversion**
4. ⏳ **Implement dual-path execution**
5. ⏳ **Test with eslint-plugin-ember rules**

## Example: What Rust Rules Will See

### Before Stripping (Full AST)

```javascript
class Counter extends Component {
  @tracked count = 0;
  increment() { this.count++; }
  <template>...</template>  // ← Glimmer node
}
```

### After Stripping (For Rust Analysis)

```javascript
class Counter extends Component {
  @tracked count = 0;
  increment() { this.count++; }
  // <template removed>  // ← Becomes comment or placeholder
}
```

### What JS Plugin Rules Will See

```javascript
class Counter extends Component {
  @tracked count = 0;
  increment() { this.count++; }
  <template>...</template>  // ← Full Glimmer AST preserved!
}
```

## Conclusion

**We are ready to proceed** with testing, but we need to implement:

1. **Glimmer node stripper** (JS-side preprocessing)
2. **Dual execution paths** (Rust + JS)
3. **Original AST pass-through** for JS plugins

The good news: **The JavaScript parts are standard ESTree/TS-ESTree** that our converter already handles. We just need to handle the Glimmer extensions appropriately.
