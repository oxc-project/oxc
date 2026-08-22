// @compilationMode(infer)
import fbt from 'fbt';

function Component(props) {
  return (
    <fbt desc="test">
      <fbt:plural count={props.count} many="items" showCount="yes">
        item
      </fbt:plural>
      <fbt:param name="nested">
        {props.showAlt ? (
          <fbt desc="nested">
            <fbt:plural count={props.altCount} many="things">
              thing
            </fbt:plural>
          </fbt>
        ) : null}
      </fbt:param>
    </fbt>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{count: 1, showAlt: true, altCount: 2}],
};
