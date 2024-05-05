# TODO

* Implement `Debug` for `Ancestor` and `AncestorWithout*` types
  * Implement intermediate `as_ref` method which creates an object of references, which debug can use.
* Improve API for `Ancestor::is_via_*`
* API to read siblings in a Vec.
* API to get which index current node is in a Vec.
* API to allow mutating other branches of AST
  * I think all that's required is to:
    * Pass `&mut TraverseCtx` to `enter_*` and `exit_*`
    * Add `parent_mut`, `ancestor_mut` methods to `TraveseCtx`
    * Add `span_mut`, `directives_mut` etc to all `*Without*` types
  * Mutable borrow on `TraverseCtx` and `Ancestor` would prevent creating more than 1 mut ref at a time
* Move `Span` to constant position in all structs, for fast lookup?
