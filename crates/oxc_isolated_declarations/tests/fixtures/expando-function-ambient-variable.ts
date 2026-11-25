const f = (): void => {};

declare namespace f {
  let property: number;
}

f.property = 1;
