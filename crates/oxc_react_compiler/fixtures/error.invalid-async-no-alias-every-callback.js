function Every() {
  let count = 0;
  const bad = () => count++;
  [0].every(async () => {
    await 0;
    bad();
  });
  return null;
}
