type Props = { users?: { id: string; name: string }[]; userId: string };

export function UserName({ users, userId }: Props) {
  return <span>{users?.find((user) => user.id === userId)!.name}</span>;
}
