import fbt from 'fbt';

function Component({a, b}) {
  return (
    <fbt desc="Description">
      <fbt:enum enum-range={['avalue1', 'avalue1']} value={a} />{' '}
      <fbt:enum enum-range={['bvalue1', 'bvalue2']} value={b} />
    </fbt>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{a: 'avalue1', b: 'bvalue2'}],
};
