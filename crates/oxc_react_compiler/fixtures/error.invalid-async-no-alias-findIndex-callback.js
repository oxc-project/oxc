function FindIndex() {
  let count = 0;
  const bad = () => count++;
  [0].findIndex(async () => {
    await 0;
    bad();
  });
  return null;
}
