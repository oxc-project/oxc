function Some() {
  let count = 0;
  const bad = () => count++;
  [0].some(async () => {
    await 0;
    bad();
  });
  return null;
}
