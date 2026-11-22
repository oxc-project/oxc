// @enablePreserveExistingMemoizationGuarantees:false
function foo(props) {
  let x, y;
  ({x, y} = {x: props.a, y: props.b});
  console.log(x); // prevent DCE from eliminating `x` altogether
  x = props.c;
  return x + y;
}

const sproutInputs = [
  [{"a":1,"b":2,"c":3}],
  [{"a":11,"b":12,"c":13}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[1]],
};
