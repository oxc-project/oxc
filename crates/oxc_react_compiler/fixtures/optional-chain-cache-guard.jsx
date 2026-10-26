export function StaticContinuation({ obj, getArg }) {
  return <span>{obj?.a.b(getArg())}</span>;
}

export function ComputedContinuation({ obj, getKey, getArg }) {
  return <span>{obj?.a[getKey()]?.(getArg())}</span>;
}

export function OptionalCallContinuation({ obj, getArg }) {
  return <span>{obj?.a.b?.(getArg()).c}</span>;
}

export function NestedContinuation({ obj, getArg }) {
  return <span>{obj.inner?.a.b(getArg())}</span>;
}

export function FullyOptional({ obj, getArg }) {
  return <span>{obj?.a?.b(getArg())}</span>;
}
