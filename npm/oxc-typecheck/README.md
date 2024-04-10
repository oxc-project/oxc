# Type Check Server for the JavaScript Oxidation Compiler

https://github.com/oxc-project/oxc/issues/2218

Proof-of-concept implementation of Rust <--> TSServer communication for `no-floating-promises` ESLint rule.
Type checker is only needed as the last step to check the type of `CallExpression`.

The way the POC works, is it copies typecheck helper implementation for `isPromiseLike` from ESLint, and exposes that as a command in `tsserver` style protocol.
To actually implement the rule, we would traverse the Rust AST until we reach expression we need to check.
And then pass the location and type of the AST node to `isPromiseLike` command to do the type check on the JS side.
This node mapping can probably be optimized to just child index access on the JS side.
