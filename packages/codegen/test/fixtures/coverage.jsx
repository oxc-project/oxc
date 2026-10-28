const value = 1;
const props = { value };
const tree = (
  <Ns.Component
    title="plain"
    quote={'contains " a quote'}
    enabled
    {...props}
  >
    text {value}
    <span>{/* comment */}child</span>
    <>
      <Leaf />
      {value + 1}
    </>
    {...items}
  </Ns.Component>
);
const namespaced = <svg:path xml:lang="en" />;
const attributeChildren = <Comp child={<span />} fragment={<><Leaf /></>} />;
const unicodeText = <span>😀</span>;
const customElement = <x-card data-id="card" />;
