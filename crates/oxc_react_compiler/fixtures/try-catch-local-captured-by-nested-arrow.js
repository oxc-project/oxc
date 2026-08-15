function Component({nodes}) {
  try {
    doSomething();
  } catch {
    const paragraphNode = createParagraphNode();
    nodes.forEach(node => paragraphNode.append(node));
    return paragraphNode;
  }
  return null;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{nodes: []}],
};
