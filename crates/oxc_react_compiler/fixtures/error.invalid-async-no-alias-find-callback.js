function Find() {
  let count = 0;
  const bad = () => count++;
  [0].find(async () => {
    await 0;
    bad();
  });
  return null;
}
