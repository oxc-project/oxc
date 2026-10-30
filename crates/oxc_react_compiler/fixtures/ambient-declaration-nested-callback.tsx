function Component() {
  declare const document: Document | undefined;
  const readEnv = () => (typeof document === "undefined" ? "server" : document.nodeName);
  return <button onClick={readEnv}>{readEnv()}</button>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
