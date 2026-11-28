// @outputMode:"lint" @panicThreshold:"none"
// Oxc compiles these forms, but upstream React Compiler records a Todo and skips
// the function, so lint mode reports the same Todos as `eslint-plugin-react-hooks`.
import { useState } from "react";

export function BigIntLiteral({ amount }: { amount: bigint }) {
  const [label] = useState("total");
  return <span title={label}>{amount === 0n ? "empty" : "paid"}</span>;
}

export function RawTaggedTemplate({ value }: { value: string }) {
  const [label] = useState("raw");
  const className = String.raw`a\b`;
  return <span title={label} className={className}>{value}</span>;
}

export function TaggedTemplateInterpolation({ value }: { value: string }) {
  const [label] = useState("interpolated");
  const className = String.raw`a-${value}`;
  return <span title={label} className={className}>{value}</span>;
}

// Upstream `main` compiles updates to captured variables, so lint mode reports
// nothing here.
export function CapturedUpdate({ items }: { items: string[] }) {
  const [label] = useState("count");
  let count = 0;
  items.forEach(() => {
    count++;
  });
  return <span title={label}>{count}</span>;
}
