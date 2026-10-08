function disposable(events, name) {
  return {
    [Symbol.dispose]() {
      events.push(name);
    },
  };
}

function asyncDisposable(events, name) {
  return {
    [Symbol.asyncDispose]: async () => {
      events.push(name);
    },
  };
}

const events = [];
let iteration = 0;
outer: for (
  using first = disposable(events, "first"), second = disposable(events, "second");
  iteration < 2;
  iteration++
) {
  expect(events).toEqual([]);
  if (iteration === 0) continue outer;
  break outer;
}
expect(events).toEqual(["second", "first"]);

const failure = new Error("initializer failed");
const caught = [];
let sawFailure = false;
try {
  for (
    using first = disposable(caught, "disposed"),
      second = (() => {
        throw failure;
      })();
    false;
    0
  ) {}
} catch (error) {
  expect(error).toBe(failure);
  sawFailure = true;
}
expect(sawFailure).toBe(true);
expect(caught).toEqual(["disposed"]);

return (async () => {
  const asyncEvents = [];
  for (
    await using first = asyncDisposable(asyncEvents, "first"),
      second = asyncDisposable(asyncEvents, "second");
    true;
    0
  ) {
    expect(asyncEvents).toEqual([]);
    break;
  }
  expect(asyncEvents).toEqual(["second", "first"]);

  const asyncFailure = new Error("async initializer failed");
  const asyncCaught = [];
  let sawAsyncFailure = false;
  try {
    for (
      await using first = asyncDisposable(asyncCaught, "disposed"),
        second = (() => {
          throw asyncFailure;
        })();
      false;
      0
    ) {}
  } catch (error) {
    expect(error).toBe(asyncFailure);
    sawAsyncFailure = true;
  }
  expect(sawAsyncFailure).toBe(true);
  expect(asyncCaught).toEqual(["disposed"]);
})();
