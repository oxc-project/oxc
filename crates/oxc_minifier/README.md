# Minifier

A JavaScript minifier has three components:

1. printer
2. mangler
3. compressor

## Printer

The printer implementation resides in `oxc_printer`. It is responsible for removing whitespace.

## Mangler

The mangler implementation is part of the `SymbolTable` residing in `oxc_semantic`.
It is responsible for shortening variables. Its algorithm should be gzip friendly.

The printer is also responsible for printing out the shortened variable names.

## Compressor

The compressor is responsible for rewriting statements and expressions for minimal text output.
[Terser](https://github.com/terser/terser) is a good place to start for learning the fundamentals.

## Terser Tests

The fixtures are copied from https://github.com/terser/terser/tree/master/test/compress
