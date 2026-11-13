import {useFragment} from 'shared-runtime';

function Component(props) {
  const user = useFragment(
    graphql`
      fragment F on User {
        name
      }
    `,
    props.user
  );
  return user.name;
}

import {graphql} from 'shared-runtime';
const sproutInputs = [
  [{user:1}],
  [{user:2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
