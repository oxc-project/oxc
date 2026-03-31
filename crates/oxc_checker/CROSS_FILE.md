# Cross-File Type Checking: Design & Plan

## Current State (as of this writing)

### What exists

- **`CheckerHost` trait** (`oxc_checker/src/host.rs`): Defines the interface between the checker and the project layer. Two methods:
  - `get_global_type(name) -> Option<TypeId>` — global type lookup (Array, String, etc.)
  - `resolve_import(from_file, module_specifier, export_name) -> Option<TypeId>` — cross-file import resolution
- **`Checker` host support** (`oxc_checker/src/checker.rs`): Has `host: Option<&'a dyn CheckerHost>`. `new_with_host()` constructor accepts a host reference. `get_global_type()` queries host first, falls back to local HashMap.
- **`oxc_project` crate** (`crates/oxc_project/src/lib.rs`): Minimal `Project` struct that implements `CheckerHost`. Currently only serves global types from lib.d.ts. `resolve_import` returns `None`.
- **`GlobalTypes`** (`oxc_checker/src/global_types.rs`): Parses lib.es5.d.ts, extracts interface declarations with properties. Filesystem walk is cached via `OnceLock` (source text found once, but still re-parsed per arena).
- **TypeArena ownership**: Caller owns the arena. `Project::new(&mut arena)` allocates global types into it, then drops the borrow. `Checker::new(semantic, &mut arena)` or `Checker::new_with_host(semantic, &mut arena, &host)` borrows it for checking. All TypeIds live in one arena.

### What doesn't exist yet

- No per-file storage in Project (no `get_semantic(path)`)
- No module resolution (specifier → file path)
- No cross-file type queries (resolve_import is a stub)
- No tsconfig.json reading or project discovery
- No compiler options struct

## How tsgo Does It

### Program interface (checker/checker.go:545-566)

The checker defines a `Program` interface with ~21 methods. Key ones:

```go
type Program interface {
    Options() *core.CompilerOptions
    SourceFiles() []*ast.SourceFile
    BindSourceFiles()
    GetSourceFile(fileName string) *ast.SourceFile
    GetResolvedModule(currentSourceFile, moduleReference, mode) *module.ResolvedModule
    GetSourceFileForResolvedModule(fileName string) *ast.SourceFile
    IsSourceFileDefaultLibrary(path) bool
    // ... more
}
```

### Checker initialization (checker/checker.go:887-894)

```go
func NewChecker(program Program) (*Checker, *sync.Mutex) {
    program.BindSourceFiles()  // eager binding of all files
    c := &Checker{}
    c.program = program
    c.compilerOptions = program.Options()
    c.files = program.SourceFiles()
    // ...
}
```

### Global symbol table (checker/checker.go:1280-1348)

Global symbols are merged from all non-module files during checker init:

```go
for _, file := range c.files {
    if !ast.IsExternalOrCommonJSModule(file) {
        c.mergeSymbolTable(c.globals, file.Locals, false, nil)
    }
}
```

Global types like `Array`, `Promise` are then resolved lazily from this merged table via memoized closures (`getGlobalTypeResolver`).

### Module resolution flow

1. Checker encounters an import → calls `c.program.GetResolvedModule(file, specifier, mode)`
2. Gets back a `ResolvedModule` with the resolved file path
3. Calls `c.program.GetSourceFileForResolvedModule(path)` to get the AST
4. Looks up the exported symbol in that file's symbol table

Resolution data is stored in `processedFiles` (compiler/fileloader.go):
```go
resolvedModules: map[tspath.Path]module.ModeAwareCache[*module.ResolvedModule]
```
Keyed by source file path, valued by `ModeAwareCache` (map from specifier+mode → resolved module).

### Host interface

`Host` embeds `ModuleSpecifierGenerationHost` which provides filesystem queries, package.json lookups, symlink handling, etc.

## Architecture Decisions Made

### Dependency direction

```
oxc_checker ──── defines CheckerHost trait
                      ▲
                      │ implements
                      │
oxc_project ──── depends on oxc_checker
```

Mirrors tsgo: checker defines the interface, compiler/project implements it. No circular deps.

### Arena ownership

Caller owns the `TypeArena`. Project borrows it during `new()` to allocate global types, then drops the borrow. Checker borrows it during checking. This avoids the Rust borrow conflict where you can't simultaneously have `&mut project.type_arena` and `&project` (as CheckerHost).

### Eager binding, lazy type checking (v1)

- Parsing + semantic: eager for v1 (all files upfront). API designed so internals can switch to lazy (OnceCell per file) without changing callers.
- Type checking: lazy/on-demand via the checker's demand-driven resolution.

## Remaining Phases

### Phase 2: Per-file storage

Project stores parsed + analyzed files. Caller provides file paths/sources, Project runs parser + semantic on each.

```rust
impl Project {
    pub fn add_file(&mut self, path: &str, source: &str, arena: &mut TypeArena) { ... }
    pub fn get_semantic(&self, path: &str) -> Option<&Semantic> { ... }
}
```

### Phase 3: Module resolution

Implement TypeScript's module resolution algorithm. Start with `node` strategy:
- Relative imports: `./foo` → `./foo.ts`, `./foo.tsx`, `./foo/index.ts`, etc.
- Non-relative: walk up `node_modules`
- `.d.ts` fallbacks

```rust
// oxc_project/src/module_resolver.rs
pub fn resolve_specifier(from_file: &Path, specifier: &str) -> Option<PathBuf> { ... }
```

### Phase 4: Cross-file type queries

Wire everything together:
1. Checker encounters unresolved import symbol (symbol_id is None)
2. Checker calls `host.resolve_import(from_file, specifier, name)`
3. Project resolves the module specifier → file path
4. Project ensures the target file is loaded + semantically analyzed
5. Project looks up the exported symbol in the target file's symbol table
6. Project runs a checker on the target file to get the export's type (cached)
7. Returns the TypeId to the original checker

The CheckerHost trait may need to grow. Possible additions:
- `get_source_file(path) -> Option<&SourceFile>` — for cross-file AST access
- `get_compiler_options() -> &CompilerOptions` — for strictness, target, etc.
- `is_default_library(path) -> bool` — to identify lib.d.ts files

## Future Parallelism Path

From checker_architecture.md §4, the recommended architecture:

1. **Phase 1 (parallel)**: Resolve declared types per-file. Per-thread arenas.
2. **Phase 2 (single-threaded)**: Merge declarations into shared read-only type environment.
3. **Phase 3 (parallel)**: Check function bodies. Per-thread arenas for temporaries, read-only access to shared environment.

This affects arena ownership: instead of one `&mut TypeArena`, each thread gets its own arena. The Project encapsulates the merge logic. The Checker API (`&mut TypeArena`) stays the same — it just receives a thread-local arena.

## oxc's Existing Module Infrastructure

- **`oxc_syntax::module_record::ModuleRecord`**: AST-level module record (import/export entries, requested modules). Uses allocator-backed types. Built during parsing.
- **`oxc_linter::module_record::ModuleRecord`**: Higher-level version with `loaded_modules: RwLock<HashMap<CompactStr, Weak<ModuleRecord>>>` for cross-file linking. Has resolved absolute paths. Not used by the checker.
- **No module resolver crate exists in oxc.** The linter's cross-file support is ad-hoc.

## Key Borrow Checker Constraints

1. **Can't hold `&mut TypeArena` and `&dyn CheckerHost` from the same struct.** Solved by having the caller own the arena separately from the Project.
2. **Can't share `&mut TypeArena` across rayon threads.** For parallel checking, need per-thread arenas or interior mutability (RefCell/Mutex). Per-thread arenas are recommended.
3. **`Semantic<'a>` borrows the allocator and source text.** Each file needs its own `Allocator` that lives at least as long as the `Semantic`. Project must own these allocators.
