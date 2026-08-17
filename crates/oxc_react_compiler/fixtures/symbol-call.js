function Component() {
  const historyKey = Symbol('history');
  return <Widget historyKey={historyKey} />;
}
