// `first` is reassigned in a memo block nested in another one. The outer block
// must restore it too, since skipping it also skips the inner block.
function Component({on, items}) {
  let first;
  let last = null;
  const list = [];
  if (on) {
    list.push(last);
    first = 'set';
  }
  for (const item of items) {
    last = item;
  }
  return <div data-first={first} data-last={last} data-n={list.length} />;
}

const ITEMS = ['a', 'b'];

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{on: true, items: ITEMS}],
  sequentialRenders: [
    {on: true, items: ITEMS},
    {on: true, items: ITEMS},
    {on: false, items: ITEMS},
    {on: false, items: ITEMS},
  ],
};
