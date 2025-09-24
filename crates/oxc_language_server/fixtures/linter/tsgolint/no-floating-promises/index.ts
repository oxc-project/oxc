const promise = new Promise((resolve, _reject) => resolve("value"));
promise;

async function returnsPromise() {
  return "value";
}

returnsPromise().then(() => {});

Promise.reject("value").catch();

Promise.reject("value").finally();

[1, 2, 3].map(async (x) => x + 1);