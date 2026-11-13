// Note that `a?.b.c` is semantically different from `(a?.b).c`
// We should codegen the correct member expressions
function Component(props) {
  let x = props?.b.c;
  let y = props?.b.c.d?.e.f.g?.h;
  return {x, y};
}

const sproutInputs = [
  [null],
  [{b:{c:{d:null}}}],
  [{b:{c:{d:{e:{f:{g:{h:42}}}}}}}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
