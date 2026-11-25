function Component(props: {value: number}) {
  let local = 0;
  let updated = 0;
  const obj = {x: props.value};
  const read = () => local;

  (local as unknown as number) = obj.x;
  ((local) as number) = obj.x;
  (local satisfies number) = local + 1;
  (local!) = local + 1;

  (obj.x as number) = local;
  (((obj.x)) as number) = local;
  (obj.x satisfies number) = local;
  (obj.x!) = local;

  (getObject()[getKey()] as number) = getValue();

  (local as number) += 1;
  (local satisfies number) -= 1;
  (local!) *= 2;

  (obj.x as number) += local;
  (obj.x satisfies number) -= local;
  (obj.x!) *= local;

  (updated as number)++;
  ++(obj.x!);

  return read() + obj.x + updated;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 0}],
};
