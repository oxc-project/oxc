// An event handler passed to JSX is not called during render, so the cache
// guard must not read `selected.id` while `selected` is null.
function Component({selected}) {
  const handleConfirm = () => {
    console.log(selected.id);
  };
  return (
    <button onClick={handleConfirm}>{selected ? selected.id : 'none'}</button>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{selected: null}],
  sequentialRenders: [
    {selected: null},
    {selected: {id: 'a'}},
    {selected: {id: 'a'}},
    {selected: null},
  ],
};
