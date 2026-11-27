// @validateNoImpureFunctionsInRender @outputMode:"lint" @compilationMode:"infer" @panicThreshold:"none"
// Zero-argument `new Date()` inside a `useMemo` callback is still render-time code:
// `drop_manual_memoization` rewrites `useMemo(fn, deps)` into a direct `fn()` call, so the
// impure call must be reported even when the IIFE pass cannot inline the callback into the
// hook body. The optional-chain dependency below defeats that inlining once an earlier
// `useMemo` has been inlined (oxc-project/oxc#29893).

// Both memos: the second callback is not inlined, yet its impure call is reported.
export function useSeed(items, range) {
  const count = useMemo(() => items.length, [items]);
  const seed = useMemo(() => range?.start ?? new Date(), [range?.start]);
  return { count, seed };
}

// A single memo inlines the callback, so the impure call is reported through the body.
export function useSeedSingleMemo(items, range) {
  const seed = useMemo(() => range?.start ?? new Date(), [range?.start]);
  return { seed };
}

// `useCallback` callbacks become plain references rather than render-time calls, so
// zero-argument `new Date()` inside them stays exempt.
export function useHandler() {
  const handler = useCallback(() => new Date(), []);
  return handler;
}
