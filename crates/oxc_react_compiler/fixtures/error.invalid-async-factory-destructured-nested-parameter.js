function Component({cond}) {
  let count = 0;
  const make = ({options: {callback}}) => async () => { await 0; callback(); };
  const run = make({options: {callback: () => count++}}); run();
  return null;
}
