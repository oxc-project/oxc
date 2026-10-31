function disposeSync(make) {
  try {
    using tryResource = make("try");
    throw new Error("catch");
  } catch (error) {
    using catchResource = make(error.message);
  } finally {
    using finallyResource = make("finally");
  }
}

async function disposeAsync(make) {
  try {
    throw new Error("catch");
  } catch {
    await using catchResource = await make("catch");
  } finally {
    await using finallyResource = await make("finally");
  }
}

function disposeDestructured(make) {
  try {
    throw { name: "catch", code: 1 };
  } catch ({ name, code }) {
    using resource = make(name, code);
  }
}
