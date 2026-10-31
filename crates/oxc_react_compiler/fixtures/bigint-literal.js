function Component(props) {
  const a = 1n;
  const b = 100n;
  const c = 0n;
  const d = 0x1fn;
  const msg = `Value is ${10n}`;
  const isZero = 0n ? 'yes' : 'no';
  const isOne = 1n ? 'yes' : 'no';
  const eq = 10n === 10n;
  const looseSame = 10n == 10n;
  const mixedEq1 = 1n == 1;
  const mixedEq2 = 1n == "1";
  const mixedEq3 = 0n == false;
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
      {looseSame ? 'same' : 'diff'}
      {mixedEq1 ? 'mixed1' : 'not-mixed1'}
      {mixedEq2 ? 'mixed2' : 'not-mixed2'}
      {mixedEq3 ? 'mixed3' : 'not-mixed3'}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
