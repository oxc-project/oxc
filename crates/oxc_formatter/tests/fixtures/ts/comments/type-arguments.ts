// A single huggable type argument stays inline unless a comment of its own ends its line
// (Prettier's `shouldInline`)
foo<A /* c1 */>();
foo<
  A // c2
>();
foo<
  // c3
  string
>(1);
new Foo<
  // c4
  T
>();
foo</* c5 */ A>();
// Comments nested inside the argument don't count
const dispatch = createEventDispatcher<{
  loaded: null; // c6
  change: string; // c7
}>();
