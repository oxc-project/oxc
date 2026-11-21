function Component({run, object}) {
  const callback = () => object.nested.value;
  return run ? callback() : null;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [],
  sequentialRenders: [
    {run: false, object: null},
    {run: true, object: {nested: {value: 42}}},
  ],
};
