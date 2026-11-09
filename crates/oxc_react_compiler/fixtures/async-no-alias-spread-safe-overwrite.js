function Component({cond}) {
  let count = 0;
  let bad = () => count++;
  const callbacks = [async () => { await 0; bad(); }];
  [0].filter(...callbacks); bad = () => null;
  return null;
}
