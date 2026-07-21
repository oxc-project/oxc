import { useRef } from "react";

export function InvalidRefAccess() {
  const ref = useRef(null);
  return <span>{ref.current}</span>;
}

export function CompilableSibling({ value }) {
  return <div>{value}</div>;
}
