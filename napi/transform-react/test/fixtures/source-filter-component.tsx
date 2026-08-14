import { useState } from "react";

export function SourceFilterComponent({ label }: { label: string }) {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>{label}: {count}</button>;
}
