function Component({cond}) {
  let count = 0;
  const bad = () => count++;
  const callbacks = [async () => { await 0; bad(); }];
  [0].map(...callbacks);
  return null;
}
