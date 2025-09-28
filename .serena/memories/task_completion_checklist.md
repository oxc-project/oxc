# Task Completion Checklist for Oxc

## Before Marking Any Task Complete

### 1. Code Quality Checks

- [ ] **Run `just fmt`** - Format all code
- [ ] **Run `just check`** - Cargo check passes
- [ ] **Run `just lint`** - All linting rules pass
- [ ] **No compilation warnings** - Clean build output

### 2. Testing Requirements

- [ ] **Run `just test`** - All unit/integration tests pass
- [ ] **Add tests for new functionality** - Don't skip test coverage
- [ ] **Update snapshots if needed** - `cargo insta review` after intended changes
- [ ] **Test affected crates specifically** - `cargo test -p <crate_name>`

### 3. Crate-Specific Updates

#### After AST Changes

- [ ] **Run `just ast`** - Update generated AST boilerplate code
- [ ] **Verify all dependent crates compile** - AST changes affect many crates

#### After Parser Changes

- [ ] **Run `just allocs`** - Update allocation snapshots
- [ ] **Run conformance tests** - `cargo coverage -- parser`
- [ ] **Check performance impact** - Parser changes affect all downstream tools

#### After Minifier Changes

- [ ] **Run `just minsize`** - Update size snapshots
- [ ] **Validate minification correctness** - Ensure output is functionally equivalent

#### After Linter Rule Changes

- [ ] **Test rule with pass/fail cases** - Use `Tester` helper
- [ ] **Generate snapshots** - `.test_and_snapshot()`
- [ ] **Register rule in appropriate category** - Don't forget rule registration

### 4. Documentation Updates

- [ ] **Update public API docs** - All public items need documentation
- [ ] **Add examples for new features** - Include usage examples
- [ ] **Update README if needed** - For user-facing changes
- [ ] **Update ARCHITECTURE.md** - For significant architectural changes

### 5. Conformance Testing

- [ ] **Run `just conformance`** - All external test suites pass
- [ ] **Check specific conformance** - `cargo coverage -- <tool>` for targeted testing
- [ ] **Update conformance fixtures** - If test expectations change

### 6. Final Integration Checks

- [ ] **Run `just ready`** - Complete CI simulation
- [ ] **Check git status** - No untracked files that should be committed
- [ ] **Verify examples work** - `just example <tool>` still functions
- [ ] **Test NAPI packages** - `pnpm test` if Node.js bindings affected

## For Major Changes

### Performance Validation

- [ ] **Run benchmarks** - `just benchmark` for performance impact
- [ ] **Profile memory usage** - Check allocation patterns
- [ ] **Test on large codebases** - Validate performance claims hold

### Breaking Changes

- [ ] **Document breaking changes** - In CHANGELOG.md
- [ ] **Update version numbers** - Follow semantic versioning
- [ ] **Coordinate with dependent projects** - If widely used APIs change

### External Dependencies

- [ ] **Update git submodules** - `just submodules` if test suites need updates
- [ ] **Check external tool compatibility** - Ensure downstream tools still work
- [ ] **Validate Node.js bindings** - NAPI packages build and test correctly

## Emergency Fixes

For critical bugs or security issues:

- [ ] **Minimal scope** - Fix only the immediate issue
- [ ] **Comprehensive testing** - Extra attention to test coverage
- [ ] **Expedited review** - Get additional eyes on the change
- [ ] **Document workarounds** - If users need immediate mitigation

## Pre-Commit Checklist (via `just ready`)

This command runs automatically but understand what it does:

1. **`git diff --exit-code --quiet`** - No uncommitted changes
2. **`typos`** - No typos in codebase
3. **`just fmt`** - Code formatting
4. **`just check`** - Compilation check
5. **`just test`** - All tests pass
6. **`just lint`** - Linting passes
7. **`just doc`** - Documentation builds
8. **`just ast`** - AST code generation up to date
9. **`git status`** - Final status check

## Common Pitfalls to Avoid

- **Forgetting to run formatters** - Always `just fmt` before committing
- **Skipping conformance tests** - These catch regressions in standards compliance
- **Not updating generated code** - AST changes require `just ast`
- **Ignoring allocation impacts** - Parser changes should run `just allocs`
- **Missing test coverage** - All new functionality needs tests
- **Forgetting NAPI implications** - Changes may affect Node.js bindings

## Success Criteria

✅ **Task is complete when:**

- All checks in this list pass
- Code follows established patterns and conventions
- Performance is maintained or improved
- Test coverage is comprehensive
- Documentation is updated
- No regressions in existing functionality
