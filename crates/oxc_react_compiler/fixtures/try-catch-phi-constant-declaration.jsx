export function Component() {
  let x = 0;
  try {
    const p = 'p';
    x = f(p);
  } catch {}
  return <div>{x}</div>;
}
