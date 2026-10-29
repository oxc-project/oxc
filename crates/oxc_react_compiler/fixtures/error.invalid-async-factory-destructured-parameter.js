function Component({cond}) {
  let count = 0;
  const make = ({callback}) => async () => { await 0; callback(); };
  const run = make({callback: () => count++}); run();
  return null;
}
