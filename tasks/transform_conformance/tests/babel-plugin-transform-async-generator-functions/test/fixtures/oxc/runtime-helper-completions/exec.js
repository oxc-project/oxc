async function* cleanup() {
  try {
    yield "body";
  } finally {
    await Promise.resolve();
    yield* [Promise.resolve("cleanup")];
  }
}

function* recover() {
  try {
    yield "body";
  } catch (error) {
    yield error.message;
  }
  return "recovered";
}

async function* delegate() {
  return yield* recover();
}

async function* directReturn() {
  return yield* ["body", "unused"];
}

return (async function () {
  const iterator = cleanup();
  expect(await iterator.next()).toEqual({ value: "body", done: false });
  expect(await iterator.return("finished")).toEqual({ value: "cleanup", done: false });
  expect(await iterator.next()).toEqual({ value: "finished", done: true });

  const recovery = delegate();
  expect(await recovery.next()).toEqual({ value: "body", done: false });
  expect(await recovery.throw(new Error("injected"))).toEqual({ value: "injected", done: false });
  expect(await recovery.next()).toEqual({ value: "recovered", done: true });

  const returned = directReturn();
  expect(await returned.next()).toEqual({ value: "body", done: false });
  expect(await returned.return("finished")).toEqual({ value: "finished", done: true });
  expect(await returned.next()).toEqual({ value: undefined, done: true });
})();
