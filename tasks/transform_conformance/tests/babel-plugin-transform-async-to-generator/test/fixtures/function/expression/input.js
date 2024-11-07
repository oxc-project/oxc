const func = async function (a, b) {
  console.log(a, await Promise.resolve());
}
setTimeout(async function (p = 0) {
  await Promise.resolve();
})