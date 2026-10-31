// The ternary's branches are constants, so only its reactive test makes the
// result change; the object built from it must depend on that result.
function Component({dark, items}) {
  let style;
  try {
    style = {color: dark ? 'white' : 'black', count: items.length};
  } catch {
    style = {};
  }
  return <span style={style}>x</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{dark: true, items: ['a']}],
  sequentialRenders: [
    {dark: true, items: ['a']},
    {dark: false, items: ['a']},
    {dark: false, items: ['a', 'b']},
    {dark: true, items: ['a', 'b']},
  ],
};
