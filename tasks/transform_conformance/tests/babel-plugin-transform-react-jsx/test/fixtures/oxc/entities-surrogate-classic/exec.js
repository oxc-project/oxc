const React = {
  createElement(type, props, ...children) {
    return { type, props, children };
  },
};
const units = (text) => text.split("").map((unit) => unit.charCodeAt(0));

const element = <div title="&#xD800;" a="&#xD83D;&#xDE00;" b="x&#xDC00;y" c="&#xFFFD;&#55357;&#56832;" d="&#xD83D;x&#xDE00;" e='"&#xD800;' f="&#xD83D;&amp;&#xDE00;">&#xD800;{"&#xD800;"}&#xD83D;&#xDE00;
  &#xD83D;
  &#xDE00;
</div>;

expect(units(element.props.title)).toEqual([0xd800]);
expect(units(element.props.a)).toEqual([0xd83d, 0xde00]);
expect(units(element.props.b)).toEqual([0x78, 0xdc00, 0x79]);
expect(units(element.props.c)).toEqual([0xfffd, 0xd83d, 0xde00]);
expect(units(element.props.d)).toEqual([0xd83d, 0x78, 0xde00]);
expect(units(element.props.e)).toEqual([0x22, 0xd800]);
expect(units(element.props.f)).toEqual([0xd83d, 0x26, 0xde00]);
expect(element.children.map(units)).toEqual([
  [0xd800],
  units("&#xD800;"),
  [0xd83d, 0xde00, 0x20, 0xd83d, 0x20, 0xde00],
]);
