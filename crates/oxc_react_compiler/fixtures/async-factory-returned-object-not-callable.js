function Component({key, cond}) {
  let count = 0;
  const cb = () => count++;
  const make = () => ({run: async () => { await 0; cb(); }});
  const result = make();
  return null;
}
