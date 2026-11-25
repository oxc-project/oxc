function Component() {
  declare var document: Document | undefined;
  const env = typeof document === "undefined" ? "server" : document.nodeName;
  return <div>{env}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
