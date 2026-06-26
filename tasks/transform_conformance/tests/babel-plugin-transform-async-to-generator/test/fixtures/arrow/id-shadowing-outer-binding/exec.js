let count = 0;
const typeNext = async () => {
  count++;
  if (count < 3) typeNext();
};
typeNext();

return new Promise(resolve => setTimeout(resolve, 50)).then(() => {
  expect(count).toBe(3);
});
