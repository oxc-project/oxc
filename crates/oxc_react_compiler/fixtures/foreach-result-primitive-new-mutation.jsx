// @enableNewMutationAliasingModel
function Component() {
  const value = [1].forEach(x => x); return {value};
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{}],
};
