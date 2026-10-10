function Component({values}) {
  let count = 0;
  values.forEach(value => {
    count += value;
    const result = count += value;
    console.log(result);
  });
  return count;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{values: [1, 2]}],
};
