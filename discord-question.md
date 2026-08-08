I'd like to extract some duplicated logic used in unicorn rules (a recursive check for whether an expression uses optional chaining) into a shared utility.
Would `utils/unicorn` be the right place for this?

The two rules currently duplicating this logic (GitHub URLs):

- https://github.com/oxc-project/oxc/blob/9e93ba6908243fc6fa2d388c8781c9ef15405fcf/crates/oxc_linter/src/rules/unicorn/prefer_dom_node_dataset.rs#L251
-

I'm also planning to use this in the `new-for-builtins` rule, so it's currently duplicated in 2 places but will become 3.
