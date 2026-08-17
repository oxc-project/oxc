import fbt from 'fbt';

/**
 * Source offsets must distinguish plurals across both <fbt:plural />
 * namespaced JSX tags and fbt.plural(...) call expressions.
 */
function useFoo({apples, bananas}) {
  return (
    <div>
      <fbt desc="Test Description">
        {fbt.param('number of apples', apples)}
        {'  '}
        {fbt.plural('apple', apples)} and
        {'  '}
        <fbt:plural name={'number of bananas'} count={bananas} showCount="yes">
          banana
        </fbt:plural>
      </fbt>
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: useFoo,
  params: [{apples: 1, bananas: 2}],
};
