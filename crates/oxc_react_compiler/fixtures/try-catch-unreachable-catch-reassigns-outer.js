// The `try` body cannot throw, so the `catch` is pruned and `x` is always `0`.
export function Component() {
  let x = 0;
  try {
    const unused = 1;
  } catch {
    x = 2;
  }
  return <div>{x}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
