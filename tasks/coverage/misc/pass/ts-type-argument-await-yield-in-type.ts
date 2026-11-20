async function f() {
  g<A<await>>(h);
  g<[await]>(h);
  g<typeof await>(h);
  let a: [await];
  let b: typeof g<await>;
  type T = G<await>;
}

function* g() {
  h<A<yield>>(i);
  let a: [yield];
}

let c: [...class: string[]];
