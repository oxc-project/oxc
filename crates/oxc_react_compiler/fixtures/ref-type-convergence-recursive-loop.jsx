export function Component({ items }) {
  let x = () => {};
  for (const item of items) {
    x = { x };
  }
  return <div onClick={x} />;
}
