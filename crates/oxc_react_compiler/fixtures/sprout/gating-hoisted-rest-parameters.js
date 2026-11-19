// @gating
const render = Component;

function Component({value}, ...rest) {
  return <span>{value + rest.length}</span>;
}

export const FIXTURE_ENTRYPOINT = {
  fn: render,
  params: [{value: 40}, 1, 2],
};
