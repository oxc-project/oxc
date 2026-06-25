# Oxc JSDoc

JSDoc comment parsing for the oxc toolchain.

## Overview

This crate provides JSDoc comment parsing functionality, extracting structured data from JSDoc comments (`/** ... */`). It handles parsing of tags, types, parameter names, and descriptions.

## Key Features

- **Lazy parsing**: JSDoc comments are parsed on first access via `OnceCell`
- **Tag extraction**: Supports `@param`, `@returns`, `@type`, `@typedef`, and arbitrary custom tags
- **Type parsing**: Extracts `{type}` expressions from tags
- **Name parsing**: Handles parameter names including optional `[name=default]` syntax
- **Comment parsing**: Strips JSDoc formatting (`*` prefixes, indentation)
- **Span tracking**: All parsed parts include source position spans
