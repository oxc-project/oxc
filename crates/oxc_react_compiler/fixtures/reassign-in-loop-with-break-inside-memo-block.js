// `first` is reassigned in a loop inside the block that builds `labels`, so the
// block must restore it when it is skipped.
function Component({items}) {
  let first = 'none';
  const labels = [];
  for (const item of items) {
    first = item;
    break;
  }
  labels.push('total');
  return <div data-first={first}>{labels}</div>;
}

const ITEMS = ['a', 'b'];

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{items: ITEMS}],
  sequentialRenders: [{items: ITEMS}, {items: ITEMS}, {items: []}, {items: []}],
};
