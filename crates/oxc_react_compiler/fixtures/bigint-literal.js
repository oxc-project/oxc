function Component(props) {
  const a = 1n;
  const b = 100n;
  const c = 0n;
  const d = 0x1fn;
  const msg = `Value is ${10n}`;
  const isZero = 0n ? 'yes' : 'no';
  const isOne = 1n ? 'yes' : 'no';
  const eq = 10n === 10n;
  return (
    <div>
      {String(a)}
      {String(b)}
      {String(c)}
      {String(d)}
      {msg}
      {isZero}
      {isOne}
      {eq ? 'equal' : 'not-equal'}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
