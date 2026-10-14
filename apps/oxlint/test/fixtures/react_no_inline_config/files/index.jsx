import { useEffect } from "react";
export function Component({ value }) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => {
    log(value);
  }, []);
  // oxlint-disable-next-line react/purity
  const rand = Math.random();
  return <div>{rand}</div>;
}
export function Other({ value }) {
  // oxlint-disable-next-line react/exhaustive-deps
  useEffect(() => {
    log(value);
  }, []);
  return null;
}
