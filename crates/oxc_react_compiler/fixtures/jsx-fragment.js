function Foo(props) {
  return (
    <>
      Hello {props.greeting}{' '}
      <div>
        <>Text</>
      </div>
    </>
  );
}

const sproutInputs = [
  [{greeting:"world"}],
  [{greeting:"React"}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
