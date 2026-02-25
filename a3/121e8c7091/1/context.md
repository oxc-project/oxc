# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Plan: Improve private identifier error recovery in parser

## Context

When `#ident` appears as a property key in invalid positions (object literals `{#x: 1}`, destructuring `const {#x} = y`, TS interface members `interface I { #x: string }`), the parser currently falls into `parse_identifier_name()` which produces an unhelpful "Unexpected token" error. No `PrivateIdentifier` AST node is created, so the semantic checker's `check_private_identifier_outside_class` ...

### Prompt 2

need to fix estree mismatches

### Prompt 3

create pr

