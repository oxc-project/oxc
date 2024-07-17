function makeBox<T>(value: T) {
  return { value };
}

type BoxFunc<T> = typeof makeBox<T>;
const makeStringBox = makeBox<string>;
