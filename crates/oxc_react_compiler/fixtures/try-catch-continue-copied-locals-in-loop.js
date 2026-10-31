// @compilationMode:"infer"
// Locals that copy each other inside a `try` in a loop used to make type
// inference take exponential time and memory.
function more(state) {
  return state.calls < 12;
}

function p(state) {
  state.calls++;
  if (state.calls % 5 === 0) {
    throw new Error('retry');
  }
  return state.calls % 3 !== 0;
}

export function Component({initial}) {
  const state = {calls: 0};
  let a = initial;
  let b = 'b';
  while (more(state)) {
    try {
      if (p(state)) a = b;
      if (p(state)) b = a;
      if (p(state)) a = b;
      if (p(state)) b = a;
    } catch {
      continue;
    }
  }
  return (
    <div>
      {a}
      {b}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{initial: 'a'}],
  sequentialRenders: [{initial: 'a'}, {initial: 'a'}, {initial: 'x'}],
};
