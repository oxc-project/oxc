const seen: string[] = [];
function dec() {
  return (_target: unknown, key: unknown) => {
    seen.push((key as Function).name);
  };
}

class C {
  @dec()
  [() => {}]() {}
}
expect(seen).toEqual([""]);
