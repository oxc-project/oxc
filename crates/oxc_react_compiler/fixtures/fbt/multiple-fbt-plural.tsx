import fbt from 'fbt';

/**
 * FBT uses Babel node `start` and `end` offsets to distinguish multiple string
 * variation arguments. The compiler must preserve those offsets on generated
 * nodes so separate plural values are not deduplicated together.
 */
function Foo({rewrites, months}) {
  return (
    <fbt desc="Test fbt description">
      <fbt:plural count={rewrites} name="number of rewrites" showCount="yes">
        rewrite
      </fbt:plural>
      to Rust ·
      <fbt:plural count={months} name="number of months" showCount="yes">
        month
      </fbt:plural>
      traveling
    </fbt>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Foo,
  params: [{rewrites: 1, months: 2}],
};
