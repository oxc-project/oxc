async function f() {
  let a: { [await: string]: number };
  let b: (await: string) => void;
  let c = d as { [await: string]: number };
  e<{ [await: string]: number }>();
}
async function* h(): AsyncGenerator<void> | { [await: string]: number; [yield: number]: number } {
  type T = { [yield: string]: number };
  let i: (yield: string) => void;
}
