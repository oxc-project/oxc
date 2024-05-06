# TODO

* Set enum discriminants for `Ancestor` so that `is_*` can use bitwise ops + `retag_stack` can write 1 byte only.
  * Compiler does not auto-optimize `matches!()` in `is_*` functions. Need to do it manually.
  * See `ancestor_type2` branch.
  * Need to handle big endian systems where bytes are in reverse order.
    * That's easy, but how to run tests on big endian? Miri?
* Implement `Debug` for `Ancestor` and `AncestorWithout*` types
  * Implement intermediate `as_ref` method which creates an object of references, which debug can use.
* Improve API for `Ancestor::is_via_*`
* API to read siblings in a Vec.
* API to get which index current node is in a Vec.
* API to allow mutating other branches of AST
  * I think all that's required is to:
    * Pass `&mut TraverseCtx` to `enter_*` and `exit_*`
    * Add `parent_mut`, `ancestor_mut` methods to `TraverseCtx`
    * Add `span_mut`, `directives_mut` etc to all `*Without*` types
  * Mutable borrow on `TraverseCtx` and `Ancestor` prevents creating more than 1 mut ref at a time
  * Mutable borrow on `TraverseCtx` unfortunately also blocks calling `ctx.alloc()`.
    * Can solve that with e.g. `ctx.ancestry.parent_mut()` + `ctx.ast.alloc()` - separate properties
      can be mut borrowed at same time.
* Move `Span` to constant position in all structs, for fast lookup?
