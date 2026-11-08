# Oxc NAPI

Shared utilities for the native Node.js bindings in `napi/`.
The individual packages provide their own exported functions and asynchronous tasks.

## Source positions

Rust source spans use UTF-8 byte offsets; JavaScript APIs use UTF-16 code-unit offsets.
`convert_utf8_to_utf16` converts positions in the program, module record, comments,
and diagnostic labels, and returns JavaScript-facing comment objects.

`get_source_type` infers the language and module kind from the filename, with optional
`lang` and `sourceType` overrides.

## Diagnostics

`OxcError` is a structured diagnostic containing severity, message, labels, optional help,
and an optional code frame. Bindings can return these objects in their results;
converting a diagnostic to `OxcError` does not throw a JavaScript exception.

## JavaScript regular expressions

`JsRegExp` converts JavaScript `RegExp` objects to and from Rust data containing the
pattern source, flags, and `lastIndex`.
