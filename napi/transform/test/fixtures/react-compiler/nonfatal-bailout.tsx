import { useRef } from "react";

type Props = {
  value: string;
};

export function InvalidRefAccess(): JSX.Element {
  const ref = useRef<HTMLDivElement>(null);
  return <span>{ref.current}</span>;
}

export function CompilableSibling({ value }: Props): JSX.Element {
  return <main>{value}</main>;
}
