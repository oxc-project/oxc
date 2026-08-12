// PruneNonEscapingScopes associates the inlined IIFE's scope with the returned
// temporary only through reassignments, while the scope's own declarations sit
// in the ternary `test` inside `try`, which is never visited as a memoization
// input — so no node is registered for the scope. Upstream fails the same
// "Expected a node for all scopes" invariant on this input; the function must
// bail out rather than crash.
export const Component = () => {
  const raw = useValue();
  return (() => {
    try {
      return check(raw) ? raw : null;
    } catch {
      return null;
    }
  })();
};

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
};
