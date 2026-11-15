type Props = { users?: { id: string; name: string }[]; userId: string };

export function UserName({ users, userId }: Props) {
  return <span>{users?.find((user) => user.id === userId)!.name}</span>;
}


const sproutInputs = [
  [{users:undefined,userId:"one"}],
  [{users:[{id:"one",name:"Ada"}],userId:"one"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return UserName(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
