debugger;

async function returnsPromise() {
  return "value";
}

returnsPromise().then(() => {});