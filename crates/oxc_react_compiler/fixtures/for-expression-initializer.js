function Component({ start, end }) {
  let index;
  let initCount = 0;
  const values = [];

  for (index = 0; index < start; index++) {
    values.push(index);
  }

  for (index = start, initCount++; index < end; index++) {
    values.push(index);
  }

  for (start < end ? values.push(-1) : values.push(-2); false;) {}
  for (start && values.push(-3); false;) {}
  const optionalValues = start ? values : null;
  for (optionalValues?.push(-4); false;) {}

  return `${initCount}:${values.join(",")}`;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{ start: 1, end: 3 }],
  sequentialRenders: [
    { start: 2, end: 5 },
    { start: 0, end: 0 },
  ],
};
