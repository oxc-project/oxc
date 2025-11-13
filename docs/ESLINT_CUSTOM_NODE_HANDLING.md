# ESLint's Approach to Custom Nodes from Custom Parsers

## How ESLint Handles Custom Nodes

ESLint's built-in rules handle custom nodes from custom parsers through a **skip-based approach**:

### Key Principles

1. **No Conversion Required**: ESLint works directly with ESTree AST - it doesn't convert to another AST format. This means custom nodes remain in the AST as-is.

2. **Visitor Keys for Traversal**: ESLint uses `visitorKeys` from `parseForESLint()` to know how to traverse custom nodes. The visitor keys tell ESLint which properties to visit when traversing each node type.

3. **Rules Only Visit Known Nodes**: Built-in rules use a visitor pattern that only visits node types they explicitly handle. If a rule doesn't have a handler for a custom node type, it simply **skips** that node - no error, no warning, just ignored.

4. **Natural Skipping**: When ESLint's traversal encounters a custom node:
   - It uses `visitorKeys` to determine which child properties to traverse
   - It recursively visits those children
   - Rules that don't know about the custom node type never get called for it
   - Rules that do know about it (via visitor keys) can process it

5. **No Stripping or Replacement**: ESLint **never** strips or replaces custom nodes. They remain in the AST throughout the linting process.

### Example: ESLint with ember-eslint-parser

```javascript
// Input: .gjs file with GlimmerTemplate node
export default class Counter extends Component {
  <template>
    <div>{{this.count}}</div>
  </template>
}

// ESLint's AST (simplified):
{
  type: "Program",
  body: [{
    type: "ExportDefaultDeclaration",
    declaration: {
      type: "ClassDeclaration",
      body: {
        type: "ClassBody",
        body: [
          {
            type: "GlimmerTemplate",  // ← Custom node stays in AST
            body: [...]
          }
        ]
      }
    }
  }]
}
```

**What happens:**
- ESLint uses `visitorKeys` from ember-eslint-parser to traverse `GlimmerTemplate`
- Built-in rules like `no-unused-vars` don't have handlers for `GlimmerTemplate`, so they skip it
- Rules traverse the children of `GlimmerTemplate` (if visitor keys specify them)
- Any standard ESTree nodes embedded in `GlimmerTemplate` (like `MemberExpression` for `this.count`) are visited by rules

### Visitor Keys Example

```javascript
// ember-eslint-parser's visitorKeys might look like:
{
  "GlimmerTemplate": ["body", "params"],
  "GlimmerPathExpression": ["path"],
  "GlimmerElementNode": ["children", "attributes"]
}

// ESLint uses these to traverse:
// - When it sees GlimmerTemplate, it visits "body" and "params" properties
// - When it sees GlimmerPathExpression, it visits "path" property
// - This allows ESLint to find embedded standard ESTree nodes
```

## Current oxlint Approach vs ESLint

### oxlint's Current Approach

1. **Strips Custom Nodes**: Custom nodes are removed before conversion to oxc AST
2. **Replaces with Placeholders**: Custom nodes are replaced with standard ESTree equivalents (e.g., `null` literals, empty statements)
3. **Heuristic Expression Extraction**: Tries to extract JavaScript expressions from custom nodes using heuristics
4. **Synthetic Methods**: Creates synthetic methods to preserve variable references for semantic analysis

**Problems:**
- Heuristic-based extraction may miss expressions
- Custom nodes are lost (can't be used by JS plugins later)
- Complex logic that may not work for all parsers
- Doesn't match ESLint's behavior

### ESLint's Approach (What We Should Match)

1. **Keeps Custom Nodes**: Custom nodes remain in the ESTree AST
2. **Uses Visitor Keys**: Traverses custom nodes using parser-provided visitor keys
3. **Natural Skipping**: Rules skip custom nodes they don't know about
4. **No Replacement**: Custom nodes are never replaced or stripped

## Proposed oxlint Approach (ESLint-Compatible)

Since oxlint converts ESTree to oxc AST (unlike ESLint which works directly with ESTree), we need a hybrid approach:

### Strategy: Skip Custom Nodes During Conversion

1. **During ESTree → oxc AST Conversion**:
   - When encountering a custom node (unknown type), **skip it** instead of erroring
   - Use `visitorKeys` to traverse custom nodes and find embedded standard ESTree nodes
   - Convert only the standard ESTree nodes found within custom nodes
   - This matches ESLint's behavior: custom nodes are "skipped" by rules (they don't exist in oxc AST)

2. **Visitor Keys Integration**:
   - Use `visitorKeys` from `parseForESLint()` to know which properties to traverse
   - Recursively search for standard ESTree nodes within custom nodes
   - Convert those standard nodes to oxc AST

3. **No Stripping Required**:
   - Don't strip custom nodes before conversion
   - Don't replace them with placeholders
   - Just skip them during conversion (they won't appear in oxc AST)

4. **Preserve Full AST for JS Plugins**:
   - Keep the full ESTree AST (with custom nodes) for JavaScript plugin rules
   - JS plugins can access custom nodes via the full AST
   - Rust rules work with oxc AST (which doesn't have custom nodes)

### Implementation Plan

#### 1. Modify ESTree Converter to Skip Custom Nodes

**Current behavior:**
```rust
// crates/oxc_linter/src/estree_converter.rs
_ => Err(ConversionError::UnsupportedNodeType {
    node_type: format!("{:?}", node_type),
    span: self.get_node_span(estree),
}),
```

**New behavior:**
```rust
// For custom nodes (EstreeNodeType::Unknown), skip them but traverse children
EstreeNodeType::Unknown(custom_type) => {
    // Use visitor keys to find embedded standard ESTree nodes
    self.convert_custom_node_with_visitor_keys(estree, visitor_keys)
}
```

#### 2. Add Visitor Keys-Based Traversal

```rust
fn convert_custom_node_with_visitor_keys(
    &mut self,
    estree: &Value,
    visitor_keys: &HashMap<String, Vec<String>>,
) -> ConversionResult<Vec<Statement<'a>>> {
    let node_type = estree.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    // Get visitor keys for this node type
    let keys_to_visit = visitor_keys.get(node_type)
        .cloned()
        .unwrap_or_default();

    let mut statements = Vec::new_in(self.builder.allocator);

    // Traverse properties specified by visitor keys
    for key in keys_to_visit {
        if let Some(value) = estree.get(key) {
            // Recursively convert children
            statements.extend(self.convert_visitor_key_value(value, key)?);
        }
    }

    Ok(statements)
}
```

#### 3. Remove Node Stripping

- Remove `stripCustomNodes()` call before conversion
- Pass full ESTree AST (with custom nodes) to converter
- Converter skips custom nodes naturally during conversion

#### 4. Update Conversion Functions

Make conversion functions return `Option` or `Vec` instead of `Result` for custom nodes:
- `convert_statement()` → Returns `Option<Statement>` (None for custom nodes)
- `convert_expression()` → Returns `Option<Expression>` (None for custom nodes)
- Arrays of statements/expressions filter out `None` values

## Benefits of This Approach

1. **ESLint-Compatible**: Matches ESLint's behavior - custom nodes are skipped, not stripped
2. **Uses Visitor Keys**: Leverages parser-provided visitor keys (standard ESLint mechanism)
3. **No Heuristics**: No need for heuristic-based expression extraction
4. **Preserves Full AST**: Full ESTree AST available for JS plugins
5. **Simpler Logic**: Skip during conversion is simpler than strip-and-replace
6. **Works for All Parsers**: Any parser that provides visitor keys will work

## Migration Path

1. **Phase 1**: Modify converter to skip custom nodes (return empty/None) instead of erroring
2. **Phase 2**: Add visitor keys-based traversal for custom nodes
3. **Phase 3**: Remove node stripping logic (no longer needed)
4. **Phase 4**: Test with multiple parsers (Ember, Vue, Svelte)

## Example: How It Would Work

```javascript
// Input: .gjs file
export default class Counter extends Component {
  <template>
    <div>{{this.count}}</div>
  </template>
}

// ESTree AST (from ember-eslint-parser):
{
  type: "Program",
  body: [{
    type: "ExportDefaultDeclaration",
    declaration: {
      type: "ClassDeclaration",
      body: {
        type: "ClassBody",
        body: [
          {
            type: "GlimmerTemplate",  // ← Custom node
            body: [{
              type: "GlimmerElementNode",
              children: [{
                type: "GlimmerMustacheStatement",
                path: {
                  type: "GlimmerPathExpression",
                  path: ["this", "count"]  // ← Embedded JS expression
                }
              }]
            }]
          }
        ]
      }
    }
  }]
}

// Visitor keys from ember-eslint-parser:
{
  "GlimmerTemplate": ["body"],
  "GlimmerElementNode": ["children"],
  "GlimmerMustacheStatement": ["path"],
  "GlimmerPathExpression": ["path"]
}

// Conversion process:
// 1. Convert Program → oxc Program
// 2. Convert ExportDefaultDeclaration → oxc ExportDefaultDeclaration
// 3. Convert ClassDeclaration → oxc ClassDeclaration
// 4. Convert ClassBody → oxc ClassBody
// 5. Encounter GlimmerTemplate (custom) → Skip, but traverse "body" (from visitor keys)
// 6. Encounter GlimmerElementNode (custom) → Skip, but traverse "children"
// 7. Encounter GlimmerMustacheStatement (custom) → Skip, but traverse "path"
// 8. Encounter GlimmerPathExpression (custom) → Skip, but traverse "path"
// 9. Find ["this", "count"] → Convert to MemberExpression(this, Identifier("count"))
// 10. Add MemberExpression to synthetic method in class body

// Result: oxc AST has MemberExpression for semantic analysis, but no GlimmerTemplate node
```

## Conclusion

ESLint's approach is elegant: **custom nodes are skipped naturally during rule execution**. oxlint should match this by:
1. Skipping custom nodes during ESTree → oxc AST conversion
2. Using visitor keys to find embedded standard ESTree nodes
3. Not stripping or replacing custom nodes (they just don't appear in oxc AST)
4. Keeping full ESTree AST for JS plugins

This matches ESLint's behavior while working within oxlint's architecture (ESTree → oxc AST conversion).

