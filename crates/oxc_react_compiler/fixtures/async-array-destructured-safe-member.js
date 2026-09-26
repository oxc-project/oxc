function Component({cond}) {
  let count = 0;
  const cb = () => count++;
  const obj = [async () => { await 0; cb(); }, () => null];
  const [,safe] = obj;
  safe();
  return null;
}
