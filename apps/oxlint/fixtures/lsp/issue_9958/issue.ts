// Test case from https://github.com/oxc-project/oxc/issues/9958
export function test() {
  let x = true;
  console.log(!!x ? 10 : 20);

  let a: {
    b?: {
      c: number;
    };
  } = {};

  console.log(a?.b?.c!);
}
