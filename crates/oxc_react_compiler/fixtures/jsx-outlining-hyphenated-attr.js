// @enableJsxOutlining
function Component({arr}) {
  return (
    <ul>
      {arr.map(item => (
        <li key={item.id} data-testid="row" aria-label={item.label}>
          <span data-id={item.id}>{item.label}</span>
        </li>
      ))}
    </ul>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{arr: [{id: 1, label: 'a'}]}],
};
