function Component() {
  const env = typeof document === "undefined" ? "server" : document.nodeName;
  declare const document: Document | undefined;
  return <div>{env}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
