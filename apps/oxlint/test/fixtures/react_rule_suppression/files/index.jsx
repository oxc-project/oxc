import { useEffect, useState } from "react";

export function A() {
  const a = Date.now();
  return <div>{a}</div>;
}

export function B() {
  // oxlint-disable-next-line react/exhaustive-deps
  const filler = 0;
  const b = Math.random() + filler;
  return <div>{b}</div>;
}

export function Form({ item }) {
  const [name, setName] = useState("");
  useEffect(() => {
    if (!item) return;
    setName(item.name);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [item?.id]);
  return <input value={name} onChange={(e) => setName(e.target.value)} />;
}
