type Props = { obj?: { get?: () => { value: number } } };

export function Value({ obj }: Props) {
  const a = obj?.get!().value;
  const b = obj?.get?.()!.value;
  return (
    <span>
      {a}
      {b}
    </span>
  );
}
