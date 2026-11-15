import fbt from 'fbt';

function Component(props) {
  return (
    <div>
      <fbt desc={'Dialog to show to user'}>
        Hello <fbt:param name="user name">{props.name}</fbt:param>
      </fbt>
      <fbt desc={'Available actions|response'}>
        <fbt:param name="actions|response">{props.actions}</fbt:param>
      </fbt>
    </div>
  );
}

const sproutInputs = [
  [{name:"Ada",actions:"Save"}],
  [{name:"Grace",actions:"Cancel"}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
