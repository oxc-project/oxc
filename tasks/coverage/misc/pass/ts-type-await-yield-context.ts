async function f() {
  let a: { [await: string]: number };
  let b: (await: string) => void;
  let c = d as { [await: string]: number };
  e<{ [await: string]: number }>();
}
async function g(): Promise<{ [await: string]: number }> {
  return {};
}
function* h(): Generator<(yield: string) => void> {
  type T = { [yield: string]: number };
  let i: (yield: string) => void;
}
