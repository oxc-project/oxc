// A JSX arrow body keeps the walk out (`assignment_chain_leaf_end`):
// when multiline, the parens are re-printed and the comment stays inside them
multiline = (a) => (<div>
  <span>very long content here to force multiline breaking of the jsx element</span>
</div> /* stays */);
// Known limitation (see DIVERGENCES.md#paren-comment-fixpoint): when the body
// fits flat the parens are dropped and the comment settles only on the second
// format; group-fit decides the parens, unknowable when the content end is chosen
flat = (a) => (<div /> /* second pass */);
