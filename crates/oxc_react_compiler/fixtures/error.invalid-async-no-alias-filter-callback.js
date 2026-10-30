function Filter() {
  let count = 0;
  const bad = () => count++;
  [0].filter(async () => {
    await 0;
    bad();
  });
  return null;
}
