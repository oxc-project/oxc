// A memo block guarded only by a short-circuited optional prefix can hide a `TypeError`: for
// `obj?.a.b(getArg())`, rendering with `obj = undefined` caches `undefined`, and rendering again
// with `obj = {}` would reuse it because `obj?.a` is still `undefined`, while the original throws
// reading `.b`. The guard must re-evaluate the chain far enough to throw at the same point.

// `.b` is read whenever `obj` is not nullish, so `obj?.a.b` is guarded too.
export function StaticContinuation({ obj, getArg }) {
  return <span>{obj?.a.b(getArg())}</span>;
}

// A computed continuation is not a property path, so the guard keeps the whole object.
export function ComputedContinuation({ obj, getKey, getArg }) {
  return <span>{obj?.a[getKey()]?.(getArg())}</span>;
}

export function OptionalCallContinuation({ obj, getArg }) {
  return <span>{obj?.a.b?.(getArg()).c}</span>;
}

export function NestedContinuation({ obj, getArg }) {
  return <span>{obj.inner?.a.b(getArg())}</span>;
}

// A fully optional chain never throws on a short-circuit, so the prefix `obj?.a` is enough.
export function FullyOptional({ obj, getArg }) {
  return <span>{obj?.a?.b(getArg())}</span>;
}

// Conditionally evaluated chains cannot hoist a throwing read above the condition, so the
// dependency is truncated before its last optional link.
export function ConditionalContinuation({ obj, getArg }) {
  return <span>{getArg() ? obj?.a.b(getArg()) : null}</span>;
}

export function ConditionalNestedContinuation({ obj, getArg }) {
  return <span>{getArg() ? obj?.a?.b.c(getArg()) : null}</span>;
}

export function ConditionalOptionalCall({ obj, getArg }) {
  return <span>{getArg() ? obj?.a.b?.(getArg()).c : null}</span>;
}

export function ConditionalComputedContinuation({ obj, getKey, getArg }) {
  return <span>{getArg() ? obj?.a[getKey()]?.(getArg()) : null}</span>;
}

export function ConditionalFullyOptional({ obj, getArg }) {
  return <span>{getArg() ? obj?.a?.b(getArg()) : null}</span>;
}
