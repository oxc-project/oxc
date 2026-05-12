let count = 0;
const typeNext = async () => {
  count++;
  if (count < 3) typeNext();
};
typeNext();
