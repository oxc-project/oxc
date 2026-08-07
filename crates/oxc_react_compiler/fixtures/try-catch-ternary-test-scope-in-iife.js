// The scope produced by `check(raw)` in the ternary test gets aligned outward to
// span the whole inlined IIFE (both `return`s become reassignments of the IIFE
// temporary + breaks out of the try). That scope's only own declaration lives
// in the ternary *test*, which PruneNonEscapingScopes never visits for
// memoization inputs, so the scope must still be registered when it is
// associated with the reassigned temporary.
function Component() {
  const raw = useValue();
  return (() => {
    try {
      return check(raw) ? raw : null;
    } catch {
      return null;
    }
  })();
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
