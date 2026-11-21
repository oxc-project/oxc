# AGENTS.md

## Adding Linting Rule

1. Use rule generator: `just new-rule <name>` (ESLint rules)
   - Or plugin-specific: `just new-ts-rule`, `just new-jest-rule`, etc.
2. Implement using visitor pattern
3. Add tests in same module
4. Register in appropriate category
