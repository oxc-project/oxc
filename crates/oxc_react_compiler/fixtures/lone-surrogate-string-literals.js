// Lone surrogate escapes in string and template literals are preserved.
function Component({text}) {
  const hasLoneHigh = text.includes('\ud800');
  const template = `<\udc00${text.length}>`;
  return (
    <div title={'\udfff'}>
      {hasLoneHigh ? 'lone surrogate' : 'clean'}
      {template.length}
      {template.charCodeAt(1)}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{text: 'a\ud800b'}],
  sequentialRenders: [{text: 'a\ud800b'}, {text: 'clean'}],
};
