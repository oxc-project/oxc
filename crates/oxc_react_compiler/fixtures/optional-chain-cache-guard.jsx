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
