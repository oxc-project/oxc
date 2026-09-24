function Component() {
  const fbt = () => 'outer';
  fbt();
  {
    const fbt = 'span';
    return <fbt desc="label">Hello</fbt>;
  }
}
