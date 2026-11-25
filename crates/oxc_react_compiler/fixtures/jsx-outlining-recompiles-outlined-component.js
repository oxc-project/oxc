// @enableJsxOutlining
function Component({items, label}) {
  return [
    items.map(item => (
      <Item label={label}>
        <Value item={item} />
      </Item>
    )),
    items.map(item => (
      <Item label={label}>
        <StrongValue item={item} />
      </Item>
    )),
  ];
}

function Item({label, children}) {
  return (
    <div>
      {label}: {children}
    </div>
  );
}

function Value({item}) {
  return item;
}

function StrongValue({item}) {
  return <strong>{item}</strong>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{items: ['one', 'two'], label: 'item'}],
};
