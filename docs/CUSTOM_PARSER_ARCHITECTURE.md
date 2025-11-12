# Custom Parser Architecture

## Overview

This document describes the technical architecture of oxc's custom parser support, implemented in January 2025.

## High-Level Architecture

### Dual-Path Execution Model

```
                    ┌─────────────────┐
                    │  Custom Parser  │
                    │ (e.g., ember)   │
                    └────────┬────────┘
                             │
                    parseForESLint()
                             │
                    ┌────────▼────────┐
                    │   ESTree AST    │
                    │ + Custom Nodes  │
                    └────────┬────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
       ┌────────▼────────┐       ┌───────▼──────┐
       │  Strip Custom   │       │  Keep Full   │
       │     Nodes       │       │     AST      │
       │   (Phase 1 ✅)  │       │  (Phase 2 ⏳) │
       └────────┬────────┘       └───────┬──────┘
                │                        │
       ┌────────▼────────┐       ┌───────▼──────┐
       │  Valid ESTree   │       │  Full AST    │
       │      AST        │       │   with       │
       └────────┬────────┘       │ Custom Nodes │
                │                └───────┬──────┘
       ┌────────▼────────┐              │
       │   Convert to    │              │
       │    oxc AST      │              │
       └────────┬────────┘              │
                │                        │
       ┌────────▼────────┐       ┌───────▼──────┐
       │   Rust Rules    │       │  JS Plugin   │
       │ ⚡ Fast & Safe  │       │    Rules     │
       └────────┬────────┘       └───────┬──────┘
                │                        │
                └────────────┬───────────┘
                             │
                    ┌────────▼────────┐
                    │  Diagnostics    │
                    └─────────────────┘
```

## Component Architecture

### 1. Configuration Layer

**Location**: `crates/oxc_linter/src/config/oxlintrc.rs`

**Purpose**: Load and validate parser configuration from `.oxlintrc.json`.

**Key Types**:
```rust
pub struct Oxlintrc {
    pub parser: Option<ParserConfig>,
    pub parser_options: Option<serde_json::Value>,
    // ...
}

pub enum ParserConfig {
    String(String),           // "ember-eslint-parser"
    Object(ParserOptions),    // { path: "...", name: "..." }
}
```

**Functionality**:
- Deserialize parser field from JSON
- Resolve relative paths to absolute
- Merge with overrides for file-specific parsers
- Pass options through to parser

### 2. Parser Store

**Location**: `crates/oxc_linter/src/external_parser_store.rs`

**Purpose**: Track loaded parsers to avoid duplicate loading.

**Key Type**:
```rust
pub struct ExternalParserStore {
    registered_parser_paths: FxHashMap<PathBuf, String>,
}
```

**Functionality**:
- Check if parser already loaded
- Register new parsers
- Look up parser names by path

### 3. External Linter Interface

**Location**: `crates/oxc_linter/src/external_linter.rs`

**Purpose**: Define callbacks for JavaScript integration.

**Key Types**:
```rust
pub type ExternalLinterLoadParserCb =
    Arc<dyn Fn(String, Option<String>) -> Result<ParserLoadResult, String>>;

pub type ExternalLinterParseWithCustomParserCb =
    Arc<dyn Fn(String, String, Option<String>) -> Result<ParseResult, String>>;

pub struct ParseResult {
    pub buffer: Vec<u8>,           // AST as JSON
    pub estree_offset: u32,        // Where JSON starts in buffer
    pub services: Option<Value>,   // Parser services
    pub scope_manager: Option<Value>,
    pub visitor_keys: Option<Value>,
}
```

**Buffer Format**:
```
┌─────────────┬─────────────────────┬──────────────┐
│ JSON Length │     JSON String     │    Offset    │
│  (4 bytes)  │   (variable size)   │   (4 bytes)  │
│  u32, LE    │   UTF-8 encoded     │   u32, LE    │
└─────────────┴─────────────────────┴──────────────┘
```

### 4. Runtime Integration

**Location**: `crates/oxc_linter/src/service/runtime.rs` (lines 960-1095)

**Purpose**: Main execution flow - decide whether to use custom or native parser.

**Flow**:
```rust
if let Some(parser_config) = resolved_config.config.parser {
    // Custom parser path
    let parse_result = external_linter.parse_with_custom_parser(...)?;

    // Store parser services for JS plugin access
    store_parser_services(parse_result.services);
    store_visitor_keys(parse_result.visitor_keys);

    // Convert ESTree to oxc AST (stripping already happened)
    let program = convert_estree_to_oxc_program(
        &parse_result.buffer,
        parse_result.estree_offset,
        source_text,
        allocator,
    )?;
} else {
    // Native oxc parser
    let ret = Parser::new(allocator, source_text, source_type).parse();
    let program = ret.program;
}

// Both paths converge here - build semantic model
let semantic = SemanticBuilder::new().build(allocator.alloc(program));
```

### 5. JavaScript Bridge

**Location**: `apps/oxlint/src/js_plugins/external_linter.rs`

**Purpose**: Create Rust functions that wrap JavaScript callbacks.

**Key Functions**:
```rust
pub fn create_external_linter(
    load_plugin: JsLoadPluginCb,
    lint_file: JsLintFileCb,
    load_parser: JsLoadParserCb,
    parse_with_custom_parser: JsParseWithCustomParserCb,
) -> ExternalLinter;

fn wrap_load_parser(cb: JsLoadParserCb) -> ExternalLinterLoadParserCb;
fn wrap_parse_with_custom_parser(cb: JsParseWithCustomParserCb)
    -> ExternalLinterParseWithCustomParserCb;
```

**Threading Model**:
- Callbacks execute on main JS thread via ThreadsafeFunction
- Rust threads block waiting for JS execution
- Uses Tokio runtime for async coordination

### 6. JavaScript Parser Interface

**Location**: `apps/oxlint/src-js/plugins/parser.ts`

**Purpose**: Load and execute custom parsers, serialize results.

**Key Functions**:

```typescript
// Load parser from path or package name
export async function loadCustomParser(
    path: string,
    packageName?: string
): Promise<string>

// Parse code and return stripped AST (for Rust rules)
export function parseWithCustomParser(
    parser: CustomParser,
    code: string,
    options?: any
): {
    buffer: Uint8Array;
    estreeOffset: number;
    services?: any;
    scopeManager?: any;
    visitorKeys?: any;
}

// Parse code and return full AST (for JS plugin rules)
export function parseWithCustomParserFull(
    parser: CustomParser,
    code: string,
    options?: any
): { /* same as above */ }
```

**Parser Loading**:
1. Dynamic import via `import(pathToFileURL(path).href)`
2. Handle default exports (function or object)
3. Handle named exports (parse, parseForESLint)
4. Validate interface compatibility
5. Cache in `registeredParsers` Map

**AST Processing Pipeline**:
```typescript
// 1. Call parser
const result = parser.parseForESLint(code, options);

// 2. Strip custom nodes (Phase 1)
const { ast: strippedAst } = stripCustomNodes(result.ast);

// 3. Add oxc hints for disambiguation
const astWithHints = addOxcHints(strippedAst);

// 4. Serialize to buffer
const { buffer, offset } = serializeEstreeToBuffer(astWithHints);

// 5. Return result
return { buffer, estreeOffset: offset, ... };
```

### 7. Node Stripper

**Location**: `apps/oxlint/src-js/plugins/strip-nodes.ts`

**Purpose**: Remove non-standard ESTree nodes from custom parser output.

**Algorithm**:
```typescript
function stripCustomNodes(ast: any, options?: StripOptions): StripResult {
    // 1. Define known types (190+ ESTree/TS-ESTree types)
    const KNOWN_ESTREE_TYPES = new Set([...]);

    // 2. Traverse AST recursively
    function traverse(node, parent, key, context) {
        // 3. Check if node type is custom
        if (node.type && !KNOWN_ESTREE_TYPES.has(node.type)) {
            // 4. Replace based on position
            if (context === 'statement' || context === 'body') {
                return createStatementReplacement(node);
            } else if (context === 'expression') {
                return createExpressionReplacement(node);
            }
        }

        // 5. Recursively process children
        for (const [key, value] of Object.entries(node)) {
            node[key] = traverse(value, node, key, inferContext(key));
        }

        return node;
    }

    return { ast: traverse(ast), stats };
}
```

**Replacement Strategy**:

| Position | Replacement | Example |
|----------|-------------|---------|
| Statement | `ExpressionStatement` with descriptive literal | `"[GlimmerTemplate removed]"` |
| Expression | `null` literal | `null` |
| Array | Filtered out | Removed from array |

**Known Types** (190+):
- **Standard ESTree (109)**: Program, statements, expressions, patterns, modules
- **TypeScript ESTree (81)**: Type annotations, interfaces, enums, decorators

### 8. ESTree to oxc Converter

**Location**: `crates/oxc_estree/src/deserialize/`

**Purpose**: Convert ESTree JSON AST to oxc's internal AST representation.

**Key Files**:
- `converter.rs` - Main conversion logic
- `statement.rs` - Statement conversions
- `expression.rs` - Expression conversions
- `pattern.rs` - Pattern conversions
- `identifier.rs` - Identifier disambiguation
- `types.rs` - ESTree type definitions

**Conversion Flow**:
```rust
pub fn convert_estree_to_oxc_program<'a>(
    buffer: &[u8],
    offset: usize,
    source_text: &'a str,
    allocator: &'a Allocator,
) -> Result<Program<'a>, ConversionError> {
    // 1. Extract JSON from buffer
    let json_str = extract_json(buffer, offset)?;

    // 2. Parse JSON to serde_json::Value
    let estree: Value = serde_json::from_str(json_str)?;

    // 3. Convert Program node
    let program = convert_program(&estree, source_text, allocator)?;

    Ok(program)
}
```

**Identifier Disambiguation**:

ESTree `Identifier` nodes are used for multiple purposes. The converter uses context and hints to disambiguate:

```rust
enum IdentifierKind {
    Binding,    // let x = 1; (variable declaration)
    Reference,  // console.log(x); (variable usage)
    Name,       // obj.prop (property name)
    Label,      // loop: while(true) (label)
}
```

**Hints**: The JavaScript parser adds `_oxc_identifierKind` properties to Identifier nodes based on parent context, making conversion more accurate.

## Data Flow

### Complete Request Flow

```
1. User runs: oxlint file.gjs
   ↓
2. Load .oxlintrc.json
   ↓
3. Find parser config: "ember-eslint-parser"
   ↓
4. Check if parser loaded (ExternalParserStore)
   ↓
5. If not loaded:
      loadCustomParser("ember-eslint-parser")
      ↓
      Resolve to node_modules/ember-eslint-parser/lib/index.js
      ↓
      Dynamic import parser module
      ↓
      Validate interface (has parse or parseForESLint)
      ↓
      Register in parser store
   ↓
6. Parse file:
      parseWithCustomParser(parser, code, options)
      ↓
      Call parser.parseForESLint(code, options)
      ↓
      Receive ESTree AST + Glimmer nodes
      ↓
      stripCustomNodes(ast)
         ↓
         Recognize 190+ standard types
         ↓
         Replace ~42 Glimmer nodes with placeholders
         ↓
         Return valid ESTree AST
      ↓
      addOxcHints(ast) - Add identifier kind hints
      ↓
      serializeEstreeToBuffer(ast)
      ↓
      Return buffer + metadata
   ↓
7. Convert to oxc AST:
      convert_estree_to_oxc_program(buffer, offset, source_text, allocator)
      ↓
      Extract JSON from buffer
      ↓
      Parse JSON to serde_json::Value
      ↓
      Recursively convert nodes
      ↓
      Use identifier hints for disambiguation
      ↓
      Return Program<'a> (oxc's internal AST)
   ↓
8. Build semantic model:
      SemanticBuilder::new().build(program)
      ↓
      Create scope tree
      ↓
      Build symbol table
      ↓
      Resolve references
      ↓
      Create CFG
   ↓
9. Run Rust linting rules:
      For each rule:
         Visit AST nodes
         Check patterns
         Report diagnostics
   ↓
10. Output results
```

## Memory Management

### Allocator Strategy

**oxc uses arena allocation** for AST nodes:

```rust
// Create arena allocator
let allocator = Allocator::default();

// All AST nodes live in the arena
let program = convert_estree_to_oxc_program(..., &allocator)?;

// No manual cleanup needed - entire arena is freed at once
drop(allocator); // All AST nodes freed
```

**Benefits**:
- Fast allocation (bump pointer)
- No individual deallocations
- Good cache locality
- Simplified lifetimes

### Buffer Management

**JavaScript to Rust transfer**:
```typescript
// JavaScript side - create buffer
const buffer = new Uint8Array(size);
// Fill with JSON
buffer.set(jsonBytes, 4);

// Transfer to Rust via NAPI
return Buffer.from(buffer).toString('base64');
```

```rust
// Rust side - decode buffer
let buffer = general_purpose::STANDARD.decode(&buffer_base64)?;

// Extract AST JSON
let json_str = std::str::from_utf8(&buffer[offset..])?;
```

**Ownership**: Buffer is owned by Rust after transfer. JavaScript doesn't retain reference.

## Threading Model

### JavaScript Integration

**Challenge**: Rust threads cannot directly call JavaScript (single-threaded).

**Solution**: NAPI ThreadsafeFunction

```rust
// Rust creates ThreadsafeFunction wrapper
let callback = ThreadsafeFunction::new(...);

// Worker thread can call JavaScript
tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
        let result = callback.call_async(args).await?;
        // Process result
    })
})
```

**Flow**:
1. Rust worker thread wants to call JS parser
2. Posts callback to JS main thread queue
3. Blocks waiting for result
4. JS main thread executes callback
5. Returns result via channel
6. Rust thread unblocks and continues

### Concurrency Safety

**Parser Store**: `Arc<ExternalParserStore>` - immutable after creation, safe to share

**Parser Services**: `Arc<Mutex<HashMap>>` - mutable, requires lock

**Visitor Keys**: `Arc<Mutex<HashMap>>` - mutable, requires lock

## Error Handling

### Error Types

```rust
// Parser loading errors
pub enum ParserLoadError {
    NotFound(String),
    InvalidInterface(String),
    LoadFailed(String),
}

// Parse errors
pub enum ParseError {
    SyntaxError { message: String, line: u32, column: u32 },
    ParserCrashed(String),
    InvalidAST(String),
}

// Conversion errors
pub enum ConversionError {
    InvalidNodeType(String),
    MissingField { node_type: String, field: String },
    InvalidValue { path: String, expected: String, got: String },
}
```

### Error Propagation

```rust
// Errors bubble up as OxcDiagnostic
fn lint_file(...) -> Result<(), Vec<OxcDiagnostic>> {
    let parse_result = parse_with_custom_parser(...)
        .map_err(|e| vec![OxcDiagnostic::error(format!("Parse failed: {}", e))])?;

    let program = convert_estree_to_oxc_program(...)
        .map_err(|e| vec![OxcDiagnostic::error(format!("Conversion failed: {}", e))])?;

    Ok(())
}
```

## Performance Characteristics

### Node Stripping Overhead

**Benchmark** (Ember GJS/GTS files):
- Parse: ~5-10ms (ember-eslint-parser)
- Strip: ~1-2ms (JavaScript traversal)
- Serialize: ~0.5-1ms (JSON.stringify)
- **Total overhead**: ~6-13ms per file

**Impact**: Negligible compared to linting time (~50-200ms per file)

### AST Size Reduction

| Metric | Before Stripping | After Stripping | Reduction |
|--------|------------------|-----------------|-----------|
| **GJS file** | 36,488 bytes | 16,126 bytes | 55.8% |
| **GTS file** | 58,314 bytes | 31,879 bytes | 45.3% |
| **Node count** | ~200 nodes | ~120 nodes | ~40% |

**Benefits**:
- Faster JSON parsing in Rust
- Less memory for AST storage
- Quicker traversal

### Memory Usage

**Per-file overhead**:
- Original AST: ~35-60 KB (JSON)
- Stripped AST: ~15-30 KB (JSON)
- oxc AST: ~10-20 KB (arena-allocated)
- Total: ~25-50 KB per file

**For 1000 files**: ~25-50 MB total (acceptable)

## Testing Strategy

### Unit Tests

**Location**: `apps/oxlint/src-js/plugins/*.test.ts`

Test individual components:
- Parser loading
- Node stripping
- Serialization

### Integration Tests

**Location**: `crates/oxc_linter/tests/ember_parser_integration.rs`

Test end-to-end flow:
```rust
#[test]
fn test_ember_gjs_stripped_ast_is_valid_estree() {
    let json = read_stripped_ast("sample.gjs.stripped.ast.json");
    let ast: Value = serde_json::from_str(&json).unwrap();

    assert_eq!(ast["type"], "Program");
    assert!(!ast.to_string().contains("\"type\":\"Glimmer"));
}
```

### Conformance Tests

**Location**: `tests/ember-parser-test/`

Real-world test files:
- `sample.gjs` - Ember Glimmer JavaScript
- `sample.gts` - Ember Glimmer TypeScript

Validation scripts:
- `parse-sample.js` - Examine parser output
- `test-stripper.js` - Test node stripper
- `strip-custom-nodes.js` - Stripper implementation (reference)

## Future Optimizations

### Phase 2: Full AST Pass-Through

**Goal**: Enable JS plugin rules to see custom nodes.

**Implementation**:
```typescript
// In parseWithCustomParserWrapper
if (needsFullAst) {
    // Store unstripped AST
    fullAstStore.set(filePath, result.ast);

    // Still return stripped version for Rust
    return stripAndSerialize(result.ast);
}
```

### Phase 3: Binary AST Format

**Current**: JSON serialization (~40% of parse time)

**Future**: Binary format
- Use MessagePack or custom binary format
- 2-3x faster serialization
- 50%+ smaller size

### Phase 4: Parser Caching

**Current**: Parser loaded once, ASTs parsed every time

**Future**: Cache parsed ASTs
- Hash file content
- Store AST in cache
- Invalidate on file change
- 10x faster for unchanged files

## Security Considerations

### Parser Safety

**Threat**: Malicious parser could execute arbitrary code.

**Mitigation**:
- Parsers run in same process (no sandboxing)
- Trust model same as ESLint
- User explicitly configures parser
- Future: optional parser sandboxing

### AST Validation

**Threat**: Malicious AST could crash converter or trigger vulnerabilities.

**Mitigation**:
- Validate node types before conversion
- Bounds checking on arrays
- Null checking on required fields
- Graceful error handling

### Buffer Security

**Threat**: Buffer overflow when parsing AST JSON.

**Mitigation**:
- Rust's safe UTF-8 parsing
- Length checks on buffer access
- serde_json validates JSON structure

## Debugging

### Enable Verbose Logging

```bash
RUST_LOG=oxc_linter=debug cargo run --bin oxlint -- file.gjs
```

### Inspect AST

```typescript
// In parser.ts, add logging:
console.log(JSON.stringify(result.ast, null, 2));
```

### Test Stripper Standalone

```bash
cd tests/ember-parser-test
node test-stripper.js
# Outputs detailed stripping statistics
```

### Debug Conversion

```rust
// In converter.rs, add:
dbg!(&estree_node);
dbg!(&oxc_node);
```

## References

### Code Locations

- Configuration: `crates/oxc_linter/src/config/`
- Parser Store: `crates/oxc_linter/src/external_parser_store.rs`
- Runtime: `crates/oxc_linter/src/service/runtime.rs`
- JavaScript Bridge: `apps/oxlint/src/js_plugins/external_linter.rs`
- Parser Interface: `apps/oxlint/src-js/plugins/parser.ts`
- Node Stripper: `apps/oxlint/src-js/plugins/strip-nodes.ts`
- ESTree Converter: `crates/oxc_estree/src/deserialize/`

### External Specifications

- [ESTree](https://github.com/estree/estree) - JavaScript AST specification
- [TS-ESTree](https://typescript-eslint.io/packages/typescript-estree) - TypeScript extensions
- [ESLint Parser API](https://eslint.org/docs/latest/extend/custom-parsers) - Parser interface
- [NAPI-RS](https://napi.rs/) - Rust-Node.js bindings

---

**Author**: Oxc Team
**Date**: January 2025
**Status**: Phase 1 Complete
**Version**: 1.0
