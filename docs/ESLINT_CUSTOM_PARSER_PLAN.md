# ESLint Custom Parser Support Implementation Plan

## Overview

This document outlines the plan for adding ESLint custom parser support to oxlint. This will allow users to use custom parsers (like `@typescript-eslint/parser`, `@babel/eslint-parser`, etc.) that conform to [ESLint's custom parser specification](https://eslint.org/docs/latest/extend/custom-parsers).

## Current Architecture

### How oxlint currently works:

1. **Rust-side parsing**: Files are parsed using `oxc_parser` (Rust)
   - Location: `crates/oxc_linter/src/service/runtime.rs::process_source_section()`
   - Produces: oxc's native AST format (`Program<'a>`)

2. **Semantic analysis**: AST is analyzed to build scopes, symbols, and node indexes
   - Location: `SemanticBuilder::build()`
   - Produces: `Semantic<'a>` object

3. **Linting**: Rules run on AST nodes via `Semantic::nodes()`
   - Location: `crates/oxc_linter/src/lib.rs::run_with_disable_directives()`
   - Rules receive: `AstNode<'a>` and `LintContext<'a>`

4. **JavaScript plugin support**: Already exists for custom rules
   - Location: `apps/oxlint/src/js_plugins/`
   - Uses shared memory buffer to pass AST to JS

### What needs to change:

- Add parser configuration to `Oxlintrc`
- Load custom parsers (JavaScript modules)
- Call custom parser before oxc parser
- Convert ESTree AST → oxc AST
- Handle `parseForESLint()` return values
- Continue with normal semantic analysis and linting

## ESLint Custom Parser Specification

### Parser Interface

Custom parsers can implement either:

1. **`parse(code, options)`** - Returns just the AST
2. **`parseForESLint(code, options)`** - Returns object with:
   - `ast`: ESTree AST
   - `services`: Parser-dependent services (e.g., type checker)
   - `scopeManager`: Custom scope analysis (optional)
   - `visitorKeys`: Custom AST traversal keys (optional)

### ESTree AST Requirements

All nodes must have:

- **`range`**: `[number, number]` - Character offsets in source code
- **`loc`**: `SourceLocation` - Line/column information (not null)
- **`parent`**: Writable property (ESLint sets this during traversal)

Program node must have:

- **`tokens`**: Array of tokens (sorted by `range[0]`)
- **`comments`**: Array of comment tokens (sorted by `range[0]`)

Literal nodes must have:

- **`raw`**: Source code of the literal

## Implementation Components

### 1. Configuration Support

**Location**: `crates/oxc_linter/src/config/oxlintrc.rs`

Add parser configuration fields:

```rust
pub struct Oxlintrc {
    // ... existing fields ...

    /// Custom parser configuration (ESLint-compatible)
    /// Can be:
    /// - String: npm package name (e.g., "@typescript-eslint/parser")
    /// - Object: { "path": "./custom-parser.js" }
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parser: Option<ParserConfig>,

    /// Parser options passed to custom parser
    /// These are parser-specific options (e.g., ecmaVersion, sourceType)
    #[serde(rename = "parserOptions", skip_serializing_if = "Option::is_none")]
    pub parser_options: Option<serde_json::Value>,
}
```

**Sticking Point**: Need to handle both string (package name) and object (path) formats, similar to how `jsPlugins` works.

### 2. Custom Parser Loader

**Location**: `crates/oxc_linter/src/config/parser.rs` (new file)

Create a parser store similar to `ExternalPluginStore`:

```rust
pub struct CustomParser {
    // JavaScript function handles
    pub parse: js_sys::Function,
    pub parse_for_eslint: Option<js_sys::Function>,
    pub meta: Option<ParserMeta>,
}

pub struct ParserStore {
    parsers: FxHashMap<PathBuf, CustomParser>,
}

impl ParserStore {
    pub fn load_parser(&mut self, path: &Path, package_name: Option<&str>)
        -> Result<CustomParser, LoadError>;

    pub fn get_parser(&self, path: &Path) -> Option<&CustomParser>;
}
```

**Sticking Point**: Need to integrate with existing JavaScript plugin loading infrastructure. May need to extend `ExternalLinter` or create parallel system.

### 3. ESTree → oxc AST Converter

**Location**: `crates/oxc_estree/src/deserialize/mod.rs` (new module)

This is the **MOST COMPLEX** component. Create a deserializer:

```rust
pub fn estree_to_oxc_program(
    estree_json: &str,
    source_text: &str,
    allocator: &Allocator,
) -> Result<Program<'a>, DeserializeError> {
    // 1. Parse ESTree JSON to structured format
    // 2. Convert ESTree nodes to oxc AST nodes
    // 3. Allocate all nodes via allocator
    // 4. Build proper node relationships
    // 5. Handle tokens and comments
}
```

**CRITICAL STICKING POINTS**:

#### a) Identifier Disambiguation

- **Problem**: ESTree has generic `Identifier` node
- **oxc has**: `BindingIdentifier`, `IdentifierReference`, `IdentifierName`, `LabelIdentifier`
- **Solution**: Need context-aware conversion:
  - In `VariableDeclarator.id` → `BindingIdentifier`
  - In `MemberExpression.property` → `IdentifierName`
  - In `CallExpression.callee` → `IdentifierReference`
  - In `LabeledStatement.label` → `LabelIdentifier`
- **Challenge**: Some contexts are ambiguous, may need to default to `IdentifierReference`

#### b) Assignment Targets vs Patterns

- **Problem**: ESTree uses `Pattern` for both binding and assignment
- **oxc has**: `AssignmentTarget` (distinct from `Pattern`)
- **Solution**: Check parent context:
  - `AssignmentExpression.left` → `AssignmentTarget`
  - `VariableDeclarator.id` → `Pattern`
- **Challenge**: Nested patterns (e.g., `[a, b] = [1, 2]`) need recursive conversion

#### c) Literal Types

- **Problem**: ESTree has generic `Literal` with `value` and `raw`
- **oxc has**: `BooleanLiteral`, `NumericLiteral`, `StringLiteral`, `BigIntLiteral`, `NullLiteral`, `RegExpLiteral`
- **Solution**: Inspect `value` type to determine which literal type
- **Challenge**: Need to preserve `raw` value correctly

#### d) Parenthesized Expressions

- **Problem**: ESTree includes `ParenthesizedExpression` wrapper
- **oxc**: May elide these or handle differently
- **Solution**: Unwrap `ParenthesizedExpression` and use `expression` property, or preserve as needed
- **Challenge**: May lose information about parentheses in source

#### e) Source Position Conversion

- **Problem**: ESTree uses `range: [number, number]` (character offsets)
- **oxc uses**: `Span { start: usize, end: usize }` (byte positions)
- **Solution**: Convert character offsets to byte offsets
  ```rust
  fn char_offset_to_byte_offset(source_text: &str, char_offset: usize) -> usize {
      source_text.char_indices()
          .nth(char_offset)
          .map(|(byte_offset, _)| byte_offset)
          .unwrap_or(source_text.len())
  }
  ```
- **Challenge**: UTF-8 encoding means character offsets ≠ byte offsets. Need efficient conversion.

#### f) Comments and Tokens

- **Problem**: ESTree attaches comments to nodes, oxc stores them separately
- **Solution**: Extract `Program.comments` and `Program.tokens`, convert to oxc format
- **Challenge**: Need to map comment positions correctly, handle comment attachment to nodes

#### g) TypeScript-Specific Nodes

- **Problem**: TS-ESTree has different node types than ESTree
- **Solution**: Handle both ESTree and TS-ESTree formats
- **Challenge**: TypeScript AST is more complex, may need separate conversion logic

#### h) Node Allocation

- **Problem**: All nodes must be allocated via `oxc_allocator`
- **Solution**: Use `allocator.alloc()` for each node during conversion
- **Challenge**: Need to track allocated nodes, handle circular references, ensure proper lifetimes

### 4. Integration into Lint Service

**Location**: `crates/oxc_linter/src/service/runtime.rs`

Modify `process_source_section` to check for custom parser:

```rust
fn process_source_section<'a>(
    &self,
    path: &Path,
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    check_syntax_errors: bool,
) -> Result<(ResolvedModuleRecord, Semantic<'a>), Vec<OxcDiagnostic>> {
    // Check if custom parser is configured for this file
    if let Some(custom_parser) = self.config.get_custom_parser(path) {
        return self.process_with_custom_parser(
            path, allocator, source_text, source_type,
            check_syntax_errors, custom_parser
        );
    }

    // Otherwise use normal oxc parser
    let ret = Parser::new(allocator, source_text, source_type)
        // ... existing code ...
}

fn process_with_custom_parser<'a>(
    &self,
    path: &Path,
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
    check_syntax_errors: bool,
    parser: &CustomParser,
) -> Result<(ResolvedModuleRecord, Semantic<'a>), Vec<OxcDiagnostic>> {
    // 1. Call custom parser (JS side)
    let estree_result = self.call_custom_parser(
        parser, source_text, path, parser_options
    )?;

    // 2. Convert ESTree → oxc AST
    let program = estree_to_oxc_program(
        &estree_result.ast_json,
        source_text,
        allocator
    )?;

    // 3. Build semantic information (same as normal path)
    let semantic_ret = SemanticBuilder::new()
        .with_cfg(true)
        .with_scope_tree_child_ids(true)
        .with_check_syntax_error(check_syntax_errors)
        .build(allocator.alloc(program));

    // 4. Handle parseForESLint return values
    // - services → store for context.sourceCode.parserServices
    // - scopeManager → decide whether to use or rebuild
    // - visitorKeys → customize AST traversal if needed

    // 5. Continue with normal module record and semantic processing
    // ... rest of processing ...
}
```

**Sticking Point**: Need to determine how to pass `parserOptions` from config to custom parser. May need to serialize and pass through JS boundary.

### 5. JavaScript/Node.js Integration

**Location**: `apps/oxlint/src-js/plugins/parser.ts` (new file)

Create parser loading and execution interface:

```typescript
export interface CustomParser {
  parse(code: string, options?: any): ESTree.Program;
  parseForESLint?(code: string, options?: any): {
    ast: ESTree.Program;
    services?: any;
    scopeManager?: any;
    visitorKeys?: any;
  };
  meta?: {
    name: string;
    version: string;
  };
}

export async function loadCustomParser(
  parserPath: string,
  packageName?: string
): Promise<CustomParser> {
  // Load parser module
  // Similar to plugin loading in load.ts
  // Return parser interface
}

export function parseWithCustomParser(
  parser: CustomParser,
  code: string,
  options: any
): string {
  // Call parser.parse() or parser.parseForESLint()
  // Serialize result to JSON
  // Return JSON string with metadata
}
```

**Sticking Point**: Need to handle async module loading. Current plugin system may need extension.

### 6. ParseForESLint Return Values

**Location**: `crates/oxc_linter/src/context/mod.rs` and related files

Handle additional return values from `parseForESLint()`:

#### a) Parser Services

- **Location**: `context.sourceCode.parserServices`
- **Solution**: Store services JSON, expose via `SourceCode` API
- **Challenge**: Services are parser-specific, need generic storage

#### b) Scope Manager

- **Problem**: Custom parser may provide its own scope analysis
- **Options**:
  1. **Ignore it**: Always rebuild with oxc's `SemanticBuilder` (recommended for Phase 1)
  2. **Use it**: Convert custom scope manager to oxc format (complex)
  3. **Hybrid**: Use for certain rules, rebuild for others
- **Recommendation**: Start with Option 1, add Option 3 later if needed

#### c) Visitor Keys

- **Problem**: Custom parser may define different AST traversal keys
- **Solution**: Use custom keys if provided, otherwise use defaults
- **Challenge**: May affect rule traversal, need to ensure compatibility

**Sticking Point**: Need to decide whether to support custom scope managers or always rebuild. Rebuilding is simpler but may lose parser-specific optimizations.

## Potential Sticking Points Summary

### 🔴 CRITICAL - Must Solve

1. **ESTree → oxc AST Conversion Complexity**
   - Identifier disambiguation (context-dependent)
   - Assignment target vs pattern distinction
   - Source position conversion (char → byte offsets)
   - Node allocation and lifetime management
   - **Impact**: Core functionality, affects all custom parsers
   - **Effort**: High (weeks of work)

2. **Parser Loading Integration**
   - How to load JavaScript modules from Rust
   - Integration with existing `ExternalLinter` system
   - Async module loading handling
   - **Impact**: Blocks implementation
   - **Effort**: Medium (few days)

### 🟡 MODERATE - Important but Workable

3. **TypeScript AST Differences**
   - TS-ESTree format vs ESTree format
   - TypeScript-specific node types
   - **Impact**: Won't support TypeScript parsers initially
   - **Effort**: Medium (additional week)

4. **Comments and Tokens Handling**
   - ESTree comment attachment vs oxc separate storage
   - Token position mapping
   - **Impact**: May affect comment-aware rules
   - **Effort**: Low-Medium (few days)

5. **Parser Services Exposure**
   - Generic storage for parser-specific data
   - Access via `context.sourceCode.parserServices`
   - **Impact**: Some rules may need parser services
   - **Effort**: Low (few days)

### 🟢 LOW - Nice to Have

6. **Custom Scope Manager Support**
   - Whether to use parser-provided scope manager
   - Conversion to oxc format
   - **Impact**: Performance optimization, not required
   - **Effort**: High (can defer to Phase 2)

7. **Performance Optimization**
   - AST caching
   - Conversion optimization
   - **Impact**: Performance, not correctness
   - **Effort**: Medium (can iterate)

## Phased Implementation Approach

### Phase 1: Basic Parser Support (MVP)

**Goal**: Support basic custom parsers with `parse()` method

1. ✅ Add parser configuration to `Oxlintrc`
2. ✅ Create basic parser loader (JS side)
3. ✅ Call custom parser before oxc parser
4. ✅ Implement minimal ESTree → oxc AST converter
   - Handle common node types (Identifier, Literal, etc.)
   - Basic identifier disambiguation
   - Simple pattern/assignment target handling
5. ✅ Continue with normal semantic analysis
6. ✅ Basic error handling

**Timeline**: 2-3 weeks

**Limitations**:

- May not handle all ESTree node types
- TypeScript parsers may not work fully
- Custom scope manager not supported
- Parser services not exposed

### Phase 2: Full Compatibility

**Goal**: Full ESLint custom parser compatibility

1. ✅ Complete ESTree → oxc AST converter
   - All node types
   - TypeScript support
   - Edge cases
2. ✅ Support `parseForESLint()` method
3. ✅ Handle `services`, `scopeManager`, `visitorKeys`
4. ✅ Expose `parserServices` to rules
5. ✅ Comprehensive testing

**Timeline**: 3-4 weeks

### Phase 3: Optimization

**Goal**: Performance and polish

1. ✅ AST caching
2. ✅ Conversion optimization
3. ✅ Parser-specific optimizations
4. ✅ Documentation

**Timeline**: 1-2 weeks

## Example Usage

Once implemented, users could configure:

```json
{
  "parser": "@typescript-eslint/parser",
  "parserOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module",
    "project": "./tsconfig.json"
  },
  "rules": {
    "no-console": "error"
  }
}
```

Or with a local parser:

```json
{
  "parser": "./my-custom-parser.js",
  "parserOptions": {
    "customOption": true
  }
}
```

## Testing Strategy

1. **Unit tests**: ESTree → oxc AST conversion for each node type
2. **Integration tests**: End-to-end with popular parsers:
   - `@typescript-eslint/parser`
   - `@babel/eslint-parser`
   - `vue-eslint-parser`
3. **Conformance tests**: Compare results with ESLint
4. **Edge case tests**: Ambiguous contexts, malformed ASTs, etc.

## Derisking the ESTree → oxc AST Conversion

This is the **most critical and complex** part of the implementation. Here are strategies to reduce risk:

### Strategy 1: Round-Trip Testing (Leverage Existing Infrastructure)

**Key Insight**: oxc already has comprehensive round-trip testing for ESTree serialization!

**Location**: `tasks/coverage/src/tools/estree.rs`

**Approach**:

1. Use existing oxc parser to generate oxc AST
2. Serialize to ESTree JSON (already tested and working)
3. Use this as **golden reference** for deserialization
4. Implement deserializer and verify: `oxc AST → ESTree JSON → oxc AST → compare`

**Implementation**:

```rust
// In tests or examples
fn test_round_trip(source_code: &str) {
    // Step 1: Parse with oxc parser → get oxc AST
    let allocator = Allocator::new();
    let ret = Parser::new(&allocator, source_code, source_type).parse();
    let original_program = ret.program;

    // Step 2: Serialize to ESTree JSON (existing, tested code)
    let estree_json = original_program.to_pretty_estree_js_json(false);

    // Step 3: Deserialize ESTree JSON → oxc AST (new code)
    let allocator2 = Allocator::new();
    let converted_program = estree_to_oxc_program(&estree_json, source_code, &allocator2)?;

    // Step 4: Compare structure (not exact equality due to allocations)
    // Compare by serializing both and comparing JSON
    let original_json = original_program.to_pretty_estree_js_json(false);
    let converted_json = converted_program.to_pretty_estree_js_json(false);
    assert_eq!(original_json, converted_json);
}
```

**Benefits**:

- Uses existing, well-tested serialization code
- Provides automatic test cases from existing test262/acorn tests
- Can catch deserialization bugs immediately
- Validates correctness incrementally

### Strategy 2: Incremental Node Type Support

**Approach**: Start with simplest, most common node types, add complexity gradually.

**Priority Order**:

1. **Phase 1A**: Basic literals and primitives
   - `BooleanLiteral`, `NumericLiteral`, `StringLiteral`, `NullLiteral`
   - Simple, no context needed
   - Test with: `const x = 42; const y = "hello";`

2. **Phase 1B**: Simple expressions
   - `BinaryExpression`, `UnaryExpression`, `LogicalExpression`
   - `MemberExpression`, `CallExpression`
   - Test with: `a + b`, `obj.prop`, `func()`

3. **Phase 1C**: Statements
   - `ExpressionStatement`, `VariableDeclaration`
   - `ReturnStatement`, `IfStatement`
   - Test with: `let x = 1;`, `if (x) return;`

4. **Phase 1D**: Identifier disambiguation (context-aware)
   - Start with conservative defaults (IdentifierReference)
   - Add context checking for common cases
   - Test incrementally

5. **Phase 2**: Complex structures
   - Functions, classes, patterns
   - Assignment targets
   - TypeScript nodes

**Implementation Pattern**:

```rust
pub fn estree_to_oxc_program(...) -> Result<Program, DeserializeError> {
    // Start with supported nodes only
    let mut supported_nodes = HashSet::new();
    supported_nodes.insert("Literal");
    supported_nodes.insert("BinaryExpression");
    // ... add as implemented

    // Check if all nodes are supported
    if !estree_ast_has_only_supported_nodes(&estree_json, &supported_nodes)? {
        return Err(DeserializeError::UnsupportedNodeType(...));
    }

    // Convert
    convert_program(estree_json, ...)
}
```

### Strategy 3: JavaScript-Side Preprocessing

**Approach**: Do complex transformations in JavaScript before passing to Rust.

**Why**: JavaScript has better tooling for ESTree manipulation, easier to debug.

**Implementation**:

```typescript
// In apps/oxlint/src-js/plugins/parser.ts
export function preprocessEstreeAst(ast: ESTree.Program): ESTree.Program {
    // 1. Add parent pointers (if not present)
    // 2. Normalize identifiers (add hints for context)
    // 3. Validate structure
    // 4. Convert problematic nodes to simpler forms

    return normalizeAst(ast);
}

function normalizeAst(ast: ESTree.Program): ESTree.Program {
    // Use a visitor to add metadata
    estreeWalker.walk(ast, {
        enter(node, parent) {
            // Add parent pointer
            node.parent = parent;

            // Add identifier type hints
            if (node.type === 'Identifier') {
                node._oxc_hint = inferIdentifierType(node, parent);
            }
        }
    });

    return ast;
}

function inferIdentifierType(
    node: ESTree.Identifier,
    parent: ESTree.Node | null
): 'binding' | 'reference' | 'name' {
    // Heuristic-based inference
    if (!parent) return 'reference';

    // Check parent context
    if (parent.type === 'VariableDeclarator' && parent.id === node) {
        return 'binding';
    }
    if (parent.type === 'MemberExpression' && parent.property === node) {
        return 'name';
    }
    // ... more heuristics

    return 'reference'; // Safe default
}
```

**Benefits**:

- Easier to debug (JavaScript tooling)
- Can use existing ESTree utilities
- Reduces Rust code complexity
- Can iterate faster

### Strategy 4: Use Existing Test Infrastructure

**Leverage**: oxc's extensive conformance test suite

**Approach**:

1. Use existing `acorn-test262` test cases
2. For each test:
   - Parse with oxc → get ESTree JSON
   - Use this as input to deserializer
   - Verify round-trip works
3. Add new test cases for edge cases

**Implementation**:

```rust
// In crates/oxc_estree/tests/round_trip.rs
#[test]
fn test_acorn_test262_cases() {
    let test_cases = load_acorn_test_cases();
    for test_case in test_cases {
        // Round-trip test
        test_round_trip(&test_case.source_code);
    }
}
```

### Strategy 5: Conservative Identifier Disambiguation

**Problem**: ESTree `Identifier` → oxc's 4 identifier types

**Approach**: Start conservative, improve incrementally

**Phase 1**: Default to `IdentifierReference` (safest)

```rust
fn convert_identifier(estree_id: &EstreeIdentifier) -> IdentifierReference {
    // Always convert to IdentifierReference initially
    // This works for most contexts
}
```

**Phase 2**: Add context-based conversion for common cases

```rust
fn convert_identifier_with_context(
    estree_id: &EstreeIdentifier,
    parent: &EstreeNode,
    field_name: &str,
) -> IdentifierKind {
    match (parent.type, field_name) {
        ("VariableDeclarator", "id") => IdentifierKind::Binding,
        ("MemberExpression", "property") => IdentifierKind::Name,
        ("LabeledStatement", "label") => IdentifierKind::Label,
        _ => IdentifierKind::Reference, // Safe default
    }
}
```

**Phase 3**: Use JavaScript-side hints (see Strategy 3)

**Fallback**: If conversion fails, use `IdentifierReference` and let semantic analysis handle it.

### Strategy 6: Validation at Each Step

**Approach**: Validate AST structure at multiple points

**Checkpoints**:

1. **After ESTree JSON parsing**: Validate JSON structure
2. **After node conversion**: Validate node structure
3. **After allocation**: Validate memory safety
4. **Before semantic analysis**: Validate AST can be analyzed

**Implementation**:

```rust
pub fn estree_to_oxc_program(...) -> Result<Program, DeserializeError> {
    // Checkpoint 1: Validate ESTree JSON
    let estree_ast = validate_estree_json(estree_json)?;

    // Checkpoint 2: Convert with validation
    let program = convert_with_validation(estree_ast, ...)?;

    // Checkpoint 3: Validate oxc AST structure
    validate_oxc_ast_structure(&program)?;

    Ok(program)
}

fn validate_oxc_ast_structure(program: &Program) -> Result<(), ValidationError> {
    // Check:
    // - All nodes have valid spans
    // - Parent-child relationships are correct
    // - Required fields are present
    // - No circular references (except parent pointers)
    // - All nodes are properly allocated
}
```

### Strategy 7: Fallback to oxc Parser

**Approach**: If custom parser fails, fall back to oxc parser

**Implementation**:

```rust
fn process_source_section<'a>(...) -> Result<...> {
    // Try custom parser first
    if let Some(custom_parser) = self.config.get_custom_parser(path) {
        match self.process_with_custom_parser(...) {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Log warning but continue
                warn!("Custom parser failed: {}. Falling back to oxc parser.", e);
                // Fall through to oxc parser
            }
        }
    }

    // Use normal oxc parser
    // ... existing code ...
}
```

**Benefits**:

- System always works (graceful degradation)
- Users can debug parser issues
- Can compare results between parsers

### Strategy 8: Source Position Conversion Optimization

**Problem**: Character offsets → byte offsets is expensive

**Approach**: Pre-compute mapping or use efficient conversion

**Option A**: Pre-compute character-to-byte mapping

```rust
struct CharToByteMapper {
    // Pre-compute for common cases
    // For large files, use lazy computation
}

impl CharToByteMapper {
    fn char_to_byte(&self, source: &str, char_offset: usize) -> usize {
        // Fast path for ASCII
        if source.is_ascii() {
            return char_offset;
        }
        // Slow path with caching
        // ...
    }
}
```

**Option B**: Accept ESTree with byte offsets (if parser supports it)

- Some parsers can be configured to output byte offsets
- Less conversion needed

### Strategy 9: Incremental Validation with Real Parser Output

**Approach**: Test with actual custom parser output early

**Implementation**:

1. Start with `@babel/parser` (well-tested, popular)
2. Parse a few test files
3. Save ESTree JSON output
4. Use as test fixtures for deserializer
5. Expand to more parsers as deserializer improves

**Benefits**:

- Tests with real-world data
- Catches parser-specific quirks early
- Validates against actual use cases

### Strategy 10: Code Generation for Conversion

**Approach**: Generate conversion code from AST schema

**Why**: oxc already uses code generation for AST (see `tasks/ast_tools/`)

**Implementation**:

1. Define ESTree → oxc mapping in schema
2. Generate conversion code automatically
3. Reduces manual conversion code
4. Easier to maintain

**Example**:

```rust
// In schema or macro
#[estree_to_oxc(
    estree_type = "Identifier",
    oxc_type = "IdentifierReference", // or context-based
    field_mapping = {
        "name" => "name",
        "range" => "span",
    }
)]
struct IdentifierConverter;
```

## Recommended Implementation Order

1. **Week 1**: Round-trip testing infrastructure
   - Set up test framework
   - Implement basic node conversion (literals, simple expressions)
   - Validate with round-trip tests

2. **Week 2**: Core node types
   - Statements, expressions
   - Identifier conversion (conservative approach)
   - JavaScript-side preprocessing

3. **Week 3**: Complex structures
   - Patterns, assignments
   - Functions, classes
   - Validation and error handling

4. **Week 4**: Polish and optimization
   - Performance optimization
   - Edge case handling
   - Documentation

## Handling oxc AST Specificity vs ESTree

One of the core challenges in ESTree → oxc AST conversion is that **oxc AST is more specific than ESTree**. This section details how to handle each case where oxc has more granular types.

### 1. Identifier Types (Most Complex)

**ESTree**: Generic `Identifier` node
**oxc**: 4 specific types:

- `BindingIdentifier` - for variable declarations and bindings
- `IdentifierReference` - for variable references
- `IdentifierName` - for property names and labels
- `LabelIdentifier` - for statement labels

**Strategy**: Context-based disambiguation

#### Context Rules (from oxc serialization patterns):

```rust
fn convert_identifier(
    estree_id: &EstreeIdentifier,
    context: &ConversionContext,
) -> IdentifierKind {
    match (context.parent_type, context.field_name) {
        // Binding contexts
        ("VariableDeclarator", "id") => IdentifierKind::Binding,
        ("FunctionDeclaration", "id") => IdentifierKind::Binding,
        ("FunctionExpression", "id") => IdentifierKind::Binding,
        ("ClassDeclaration", "id") => IdentifierKind::Binding,
        ("ClassExpression", "id") => IdentifierKind::Binding,
        ("CatchClause", "param") => IdentifierKind::Binding, // if it's an Identifier
        ("Property", "key") if context.is_shorthand => IdentifierKind::Binding,

        // Name contexts (property names, not values)
        ("MemberExpression", "property") => IdentifierKind::Name,
        ("Property", "key") if !context.is_shorthand => IdentifierKind::Name,
        ("MethodDefinition", "key") => IdentifierKind::Name,
        ("ExportSpecifier", "exported") => IdentifierKind::Name,
        ("ImportSpecifier", "imported") => IdentifierKind::Name,

        // Label contexts
        ("LabeledStatement", "label") => IdentifierKind::Label,
        ("BreakStatement", "label") => IdentifierKind::Label,
        ("ContinueStatement", "label") => IdentifierKind::Label,

        // Default: IdentifierReference (safest fallback)
        _ => IdentifierKind::Reference,
    }
}
```

**Edge Cases**:

- **Object destructuring**: `{ a } = obj` - `a` in `Property.key` when shorthand is binding
- **Computed properties**: `{ [key]: value }` - `key` is an expression, not identifier
- **Method names**: In class methods, the key can be identifier name

**Fallback Strategy**: When context is ambiguous, default to `IdentifierReference`. This is safe because:

- Semantic analysis can still work (references are the most general)
- Some rules may miss optimizations, but won't break
- Can be improved incrementally

### 2. Pattern vs AssignmentTarget

**ESTree**: Uses `Pattern` for both binding and assignment
**oxc**: Distinguishes `Pattern` (for bindings) from `AssignmentTarget` (for assignments)

**Key Insight**: In ESTree, `AssignmentExpression.left` is a `Pattern`, but in oxc it's an `AssignmentTarget`.

**Strategy**: Context-based conversion

```rust
fn convert_pattern_or_assignment_target(
    estree_node: &EstreeNode,
    context: &ConversionContext,
) -> Result<Either<Pattern, AssignmentTarget>, ConversionError> {
    match context.parent_type {
        "AssignmentExpression" if context.field_name == "left" => {
            // This is an assignment target
            Ok(Either::Right(convert_to_assignment_target(estree_node)?))
        }
        "VariableDeclarator" if context.field_name == "id" => {
            // This is a binding pattern
            Ok(Either::Left(convert_to_pattern(estree_node)?))
        }
        "ForInStatement" | "ForOfStatement" if context.field_name == "left" => {
            // Loop variable is a binding pattern
            Ok(Either::Left(convert_to_pattern(estree_node)?))
        }
        _ => {
            // Default: try to determine from node structure
            // If it's an Identifier in assignment context, it's an AssignmentTarget
            if estree_node.type == "Identifier" && context.is_assignment_context() {
                Ok(Either::Right(convert_to_assignment_target(estree_node)?))
            } else {
                Ok(Either::Left(convert_to_pattern(estree_node)?))
            }
        }
    }
}
```

**Nested Patterns**: Both patterns and assignment targets can be nested:

- `[a, b] = [1, 2]` - Array pattern/assignment target
- `{ a, b } = obj` - Object pattern/assignment target

**Strategy**: Recursive conversion, maintaining context through the tree.

### 3. Literal Types

**ESTree**: Generic `Literal` node with `value` property
**oxc**: Specific literal types:

- `BooleanLiteral`
- `NumericLiteral`
- `StringLiteral`
- `BigIntLiteral`
- `NullLiteral`
- `RegExpLiteral`

**Strategy**: Type inspection of `value` field

```rust
fn convert_literal(estree_literal: &EstreeLiteral) -> Result<LiteralKind, ConversionError> {
    // Check value type
    match estree_literal.value {
        Value::Boolean(b) => Ok(LiteralKind::Boolean(BooleanLiteral { value: b, ... })),
        Value::Number(n) => Ok(LiteralKind::Numeric(NumericLiteral { value: n, ... })),
        Value::String(s) => Ok(LiteralKind::String(StringLiteral { value: s, ... })),
        Value::Null => Ok(LiteralKind::Null(NullLiteral { ... })),
        // BigInt and RegExp have special handling
        Value::BigInt(bi) => Ok(LiteralKind::BigInt(BigIntLiteral { value: bi, ... })),
    }

    // Check for regexp (has `regex` property)
    if estree_literal.regex.is_some() {
        return Ok(LiteralKind::RegExp(convert_regex_literal(estree_literal)?));
    }
}
```

**Important**: ESTree `Literal.raw` must be preserved - it's the source code representation.

### 4. ParenthesizedExpression Handling

**ESTree**: Includes `ParenthesizedExpression` wrapper nodes
**oxc**: Can elide these (parser option `preserve_parens`)

**Strategy**: Unwrap `ParenthesizedExpression` nodes

```rust
fn unwrap_parenthesized_expression(
    estree_node: &EstreeNode,
) -> &EstreeNode {
    if estree_node.type == "ParenthesizedExpression" {
        // Unwrap and return inner expression
        &estree_node.expression
    } else {
        estree_node
    }
}
```

**Note**: oxc's serialization removes `ParenthesizedExpression` when `preserve_parens: false`. For consistency, we should do the same during deserialization.

**Preserving Parentheses**: If needed for specific rules, we can:

- Option 1: Keep them as `ParenthesizedExpression` (if oxc supports it)
- Option 2: Store in metadata/trivia
- Option 3: Reconstruct from source text if needed

### 5. JSX Identifier Handling

**ESTree**: `JSXIdentifier` for JSX element names
**oxc**: Uses `IdentifierReference` or `ThisExpression` for JSX element names

**Strategy**: Convert `JSXIdentifier` → `IdentifierReference`

```rust
fn convert_jsx_identifier(estree_jsx_id: &EstreeJSXIdentifier) -> IdentifierReference {
    // Simple conversion - JSXIdentifier is just an identifier in JSX context
    IdentifierReference {
        span: convert_span(estree_jsx_id.range),
        name: estree_jsx_id.name,
    }
}
```

**Special Case**: If `JSXIdentifier.name == "this"`, oxc may use `ThisExpression` instead. Check oxc's serialization behavior.

### 6. MemberExpression vs StaticMemberExpression/ComputedMemberExpression

**ESTree**: `MemberExpression` with `computed: boolean`
**oxc**: `StaticMemberExpression` vs `ComputedMemberExpression`

**Strategy**: Check `computed` property

```rust
fn convert_member_expression(
    estree_member: &EstreeMemberExpression,
) -> Result<MemberExpressionKind, ConversionError> {
    if estree_member.computed {
        Ok(MemberExpressionKind::Computed(ComputedMemberExpression {
            object: convert_expression(estree_member.object)?,
            expression: convert_expression(estree_member.property)?,
            span: convert_span(estree_member.range),
        }))
    } else {
        Ok(MemberExpressionKind::Static(StaticMemberExpression {
            object: convert_expression(estree_member.object)?,
            property: convert_identifier_name(estree_member.property)?,
            span: convert_span(estree_member.range),
        }))
    }
}
```

### 7. Function Body vs Expression Body

**ESTree**: Arrow functions can have `Expression` or `BlockStatement` body
**oxc**: `ArrowFunctionExpression` has `FunctionBody` (which can be expression or block)

**Strategy**: Convert based on body type

```rust
fn convert_arrow_function_body(
    estree_body: &EstreeNode,
) -> Result<FunctionBody, ConversionError> {
    match estree_body.type {
        "BlockStatement" => {
            Ok(FunctionBody::Block(convert_block_statement(estree_body)?))
        }
        _ => {
            // Expression body - wrap in BlockStatement with ReturnStatement
            let expression = convert_expression(estree_body)?;
            Ok(FunctionBody::Expression(expression))
        }
    }
}
```

**Note**: oxc's `ArrowFunctionExpression` has `expression: bool` flag to indicate expression body.

### 8. TypeScript-Specific Conversions

**TS-ESTree**: Has additional node types not in ESTree
**oxc**: TypeScript nodes are integrated into main AST

**Strategy**: Handle TS-ESTree extensions separately

- `TSQualifiedName` → `QualifiedName` (oxc) or convert to `MemberExpression`
- `TSThisParameter` → `TSThisParameter` (oxc)
- Type annotations are preserved as `TSTypeAnnotation`

**Complexity**: TypeScript AST is significantly more complex. Consider:

- Phase 1: Support JavaScript-only ESTree
- Phase 2: Add TypeScript support
- Use existing TS-ESTree test fixtures

### Implementation Pattern: Context-Aware Conversion

Create a conversion context that tracks parent information:

```rust
struct ConversionContext {
    parent_type: Option<String>,
    parent_node: Option<*const EstreeNode>, // For complex cases
    field_name: Option<String>, // Which field this node is in
    path: Vec<String>, // Full path for debugging
}

impl ConversionContext {
    fn new() -> Self {
        Self {
            parent_type: None,
            parent_node: None,
            field_name: None,
            path: Vec::new(),
        }
    }

    fn with_parent(&self, parent_type: &str, field_name: &str) -> Self {
        let mut path = self.path.clone();
        path.push(format!("{}.{}", parent_type, field_name));
        Self {
            parent_type: Some(parent_type.to_string()),
            parent_node: self.parent_node,
            field_name: Some(field_name.to_string()),
            path,
        }
    }

    fn is_assignment_context(&self) -> bool {
        self.parent_type.as_deref() == Some("AssignmentExpression")
            && self.field_name.as_deref() == Some("left")
    }

    fn is_binding_context(&self) -> bool {
        matches!(
            (self.parent_type.as_deref(), self.field_name.as_deref()),
            (Some("VariableDeclarator"), Some("id"))
            | (Some("FunctionDeclaration"), Some("id"))
            | (Some("FunctionExpression"), Some("id"))
            // ... more cases
        )
    }
}
```

### Validation Strategy

After conversion, validate that the more specific types are correct:

```rust
fn validate_converted_ast(program: &Program) -> Result<(), ValidationError> {
    // Walk AST and validate:
    // 1. Identifiers are in correct contexts
    // 2. AssignmentTargets are only in assignment contexts
    // 3. Patterns are only in binding contexts
    // 4. All node types are valid

    // Use semantic analysis to catch issues
    let semantic = SemanticBuilder::new().build(program);
    // Semantic analysis will catch type mismatches
}
```

### Fallback Strategy

When conversion is ambiguous or fails:

1. **Identifier**: Default to `IdentifierReference` (most general)
2. **Pattern**: Default to `BindingPattern` (can be used in most contexts)
3. **Literal**: Error if type cannot be determined (shouldn't happen)
4. **Log warnings**: Record ambiguous conversions for analysis

```rust
enum ConversionWarning {
    AmbiguousIdentifier {
        name: String,
        context: String,
        used_type: IdentifierKind,
        suggested_type: Option<IdentifierKind>,
    },
    // ... more warning types
}
```

## Risk Mitigation Summary

| Risk                        | Mitigation Strategy                                       | Priority |
| --------------------------- | --------------------------------------------------------- | -------- |
| Complex conversion logic    | Incremental implementation, round-trip testing            | High     |
| Identifier disambiguation   | Context-based rules, conservative defaults, JS-side hints | High     |
| Pattern vs AssignmentTarget | Context-aware conversion, validation                      | High     |
| Source position conversion  | Pre-computation, ASCII fast path                          | Medium   |
| TypeScript support          | Defer to Phase 2, start with JS only                      | Medium   |
| Performance                 | Optimize after correctness                                | Low      |
| Edge cases                  | Extensive testing, fallback to oxc parser                 | Medium   |

## Open Questions

1. **Should we support parser caching?**
   - Pros: Performance for repeated files
   - Cons: Memory usage, complexity

2. **How to handle parser errors?**
   - Fall back to oxc parser?
   - Report error and stop?

3. **Should we support parser plugins?**
   - Some parsers have plugin systems (e.g., Babel)
   - May need to extend parser options

4. **TypeScript project references?**
   - `@typescript-eslint/parser` needs `project` option
   - May need to handle TypeScript config loading

## Related Files

- Configuration: `crates/oxc_linter/src/config/oxlintrc.rs`
- Lint service: `crates/oxc_linter/src/service/runtime.rs`
- ESTree serialization: `crates/oxc_estree/src/serialize/`
- JavaScript plugins: `apps/oxlint/src-js/plugins/`
- External linter: `apps/oxlint/src/js_plugins/external_linter.rs`
- ESTree tests: `tasks/coverage/src/tools/estree.rs`

## References

- [ESLint Custom Parser Documentation](https://eslint.org/docs/latest/extend/custom-parsers)
- [ESTree Specification](https://github.com/estree/estree)
- [TypeScript ESTree AST](https://github.com/typescript-eslint/typescript-eslint/tree/main/packages/ast-spec)
