import { useEffect, useState } from "react";
export function Component() {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const filler = 0;
  const rand = Math.random() + filler;
  return <div>{rand}</div>;
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
