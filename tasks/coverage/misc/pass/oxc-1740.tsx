export const foo = () => ({
  render: async () => {
    ReactDOM.render(
      <Bar {...config} attribute={await baz} />,
      domNode,
      () => {}
    );
  },
});
