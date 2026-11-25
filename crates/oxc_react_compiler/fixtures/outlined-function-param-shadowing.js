function Component(props) {
  const {value} = props;
  const items = props.list.filter(value => value > 0);
  return (
    <div>
      {items.length}
      {value}
    </div>
  );
}


const sproutInputs = [
  [{value:1,list:[-1,2,3]}],
  [{value:2,list:[4,0]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
