export type T = number;

function Foo(): T {
  type T = string;
  return 0;
}

const Bar = (): T => {
  type T = string;
  return 0;
}

