const A = async (a) => {
  await Promise.resolve();
}
setTimeout(async (p = 0) => {
  await Promise.resolve();
  console.log(p)
})