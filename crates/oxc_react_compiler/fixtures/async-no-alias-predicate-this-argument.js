function Component() {
  let count = 0;
  const bad = () => count++;
  const receiver = async () => {
    await 0;
    bad();
  };
  const values = [0];
  values.filter(() => true, receiver);
  values.every(() => true, receiver);
  values.some(() => true, receiver);
  values.find(() => true, receiver);
  values.findIndex(() => true, receiver);
  new Set(values).forEach(() => null, receiver);
  new Map().forEach(() => null, receiver);
  return null;
}
