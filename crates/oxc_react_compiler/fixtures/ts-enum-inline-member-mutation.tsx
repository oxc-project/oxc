function Child(props: {items: Array<number>}) {
  return <span>{props.items.length}</span>;
}

function Component(props: {items: Array<number>}) {
  enum Result {
    Length = props.items.push(1),
  }

  return <Child items={props.items} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{items: []}],
};
