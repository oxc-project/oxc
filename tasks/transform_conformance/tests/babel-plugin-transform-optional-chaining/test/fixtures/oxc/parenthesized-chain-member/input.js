const user = undefined;
console.log(((user?.profile).settings)?.theme);

function test(a, k) {
  (a?.b)[k]?.c;
  (a?.b)()?.c;
  (a?.b).c?.();
  delete (a?.b).c?.d;
  (a?.b)?.c;
  (a?.b)?.();
}

class Foo {
  #value;

  test(a) {
    return (a?.b).#value?.c;
  }
}
