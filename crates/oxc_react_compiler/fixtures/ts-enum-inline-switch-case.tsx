function Component(props: {action: string}) {
  enum Action {
    Select,
  }

  switch (props.action) {
    case String(Action.Select):
      return <span>selected</span>;
    default:
      return <span>unknown</span>;
  }
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{action: '0'}],
};
