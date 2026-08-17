function Component() {
  let value = 0;
  const read = () => value;

  (value as number)++;

  return read();
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
};
