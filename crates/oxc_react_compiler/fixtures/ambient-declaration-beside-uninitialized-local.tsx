function Component({ flag }) {
  let value;
  declare const document: Document | undefined;
  if (flag) value = typeof document === "undefined" ? "server" : document.nodeName;
  return <div>{value}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{ flag: true }],
  sequentialRenders: [{ flag: true }, { flag: false }, { flag: true }],
};
