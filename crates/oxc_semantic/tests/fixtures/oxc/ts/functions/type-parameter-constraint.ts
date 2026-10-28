export type T = number;

function Foo<U extends T>() {
  type T = string;
  return null! as U;
}

const Bar = <U extends T>() => {
  type T = string;
  return null! as U;
}
