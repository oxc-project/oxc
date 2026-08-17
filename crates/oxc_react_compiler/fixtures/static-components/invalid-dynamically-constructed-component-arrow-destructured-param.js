// @loggerTestOnly @validateStaticComponents @outputMode:"lint"
function Example() {
  const Component = ({x}) => <div>{x}</div>;
  return <Component x="value" />;
}
