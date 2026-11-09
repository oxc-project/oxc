function Component({cond}) {
  let count = 0;
  const cb = () => count++;
  const obj = [() => null, async () => { await 0; cb(); }];
  const [, ...rest] = obj;
  rest[0]();
  return null;
}
