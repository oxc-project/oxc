function Component() {
  let count = 0;
  const bad = () => count++;
  const wrapper = () => bad;
  return [0].map(wrapper);
}
