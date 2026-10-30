const DEBUG = false;

// `label` is set in a branch on `enabled` that also returns early behind a
// constant, so it still depends on `enabled`.
function Component({enabled}) {
  let label = 'off';
  if (enabled) {
    label = 'on';
    if (DEBUG) {
      return null;
    }
  }
  return <div>{label}</div>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{enabled: true}],
  sequentialRenders: [{enabled: true}, {enabled: false}, {enabled: true}],
};
