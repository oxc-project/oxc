function Component() {
  const value = [1].forEach(x => x); return <div>{value}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
